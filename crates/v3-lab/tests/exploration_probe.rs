//! Evolvability exploration probes (2026-10 run, `docs/strategy/evolvability-exploration-2026-10.md`).
//! Observation only: production mutation engine, lab evaluator and scenes,
//! no selection and nothing fed back into variation. Ignored by default; run
//! with `cargo test --release -p v3-lab --test exploration_probe -- --ignored --nocapture`.
//! Each probe prints one summary JSON line and, when `PETRI_PROBE_OUT` names a
//! file under `.bench-artifacts/`, writes one compact JSON line per child.

use std::collections::BTreeMap;
use std::io::Write;

use rand::rngs::SmallRng;
use rand::SeedableRng;
use rayon::prelude::*;
use serde_json::json;
use v3_core::contracts::InputReference;
use v3_core::creature::founder::founder_genome_with_age_gate;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use v3_core::creature::genome::{BackendDef, CreatureGenome};
use v3_core::mutation::reachability::ParentExecuted;
use v3_core::mutation::MutationEngine;
use v3_lab::arena::arena_config;
use v3_lab::eval::{evaluate_genome, Frozen, Setup};
use v3_lab::scene::{Scene, SceneSpec};

const SIZE: u16 = 64;
const FRACTION: f64 = 0.04;
const LIFETIME: u32 = 200;
const START_ENERGY: f32 = 100.0;

fn setup() -> Setup {
    Setup::new(arena_config(SIZE), START_ENERGY, LIFETIME)
}

fn scenes(seed: u64, n: usize, setup: &Setup) -> Vec<Scene> {
    let spec = SceneSpec::sparse(
        SIZE,
        FRACTION,
        setup.config.runtime.perception.vision_radius,
    );
    let mut rng = SmallRng::seed_from_u64(seed);
    (0..n)
        .map(|_| spec.draw(&mut rng).expect("feasible scene"))
        .collect()
}

fn scores(setup: &Setup, genome: &CreatureGenome, scenes: &[Scene]) -> Vec<f64> {
    scenes
        .iter()
        .map(|s| evaluate_genome(setup, genome, s).0.score)
        .collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn corr(a: &[f64], b: &[f64]) -> Option<f64> {
    let (ma, mb) = (mean(a), mean(b));
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let va: f64 = a.iter().map(|x| (x - ma).powi(2)).sum();
    let vb: f64 = b.iter().map(|y| (y - mb).powi(2)).sum();
    (va > 0.0 && vb > 0.0).then(|| cov / (va * vb).sqrt())
}

fn out_file() -> Option<std::fs::File> {
    std::env::var("PETRI_PROBE_OUT")
        .ok()
        .map(|p| std::fs::File::create(p).expect("probe out"))
}

/// E3, fixed-pool selection audit (H1). Children of the founder are scored on
/// two independent 4-scene batches (the campaign's per-generation training
/// size) and a 32-scene diagnostic bank. Reports how well a 4-scene delta
/// predicts the bank delta and what truncation on batch A buys on the bank
/// against a seeded random pick of the same size.
#[test]
#[ignore = "exploration probe; seconds-to-minutes in release"]
#[allow(clippy::type_complexity)]
fn e3_fixed_pool_selection_audit() {
    const POOLS: u64 = 8;
    const POOL: usize = 64;
    const KEEP: usize = 16;
    let setup = setup();
    let founder = founder_genome_with_age_gate(
        setup.config.population.founder_profile,
        &setup.config.energy.lifecycle,
    );
    let reachable = mesh_reachable_nodes(&founder);
    let bank = scenes(0xE3_BA_4C, 32, &setup);
    let founder_bank = scores(&setup, &founder, &bank);
    let food_types = setup.config.world.food.types.len();
    let mut rows = Vec::new();
    let mut summary_pools = Vec::new();
    for pool in 0..POOLS {
        let batch_a = scenes(0xE3_A0_00 + pool, 4, &setup);
        let batch_b = scenes(0xE3_B0_00 + pool, 4, &setup);
        let founder_a = mean(&scores(&setup, &founder, &batch_a));
        let founder_b = mean(&scores(&setup, &founder, &batch_b));
        // The lab's frozen record: the parent's last training scene.
        let frozen: Frozen = evaluate_genome(&setup, &founder, batch_a.last().unwrap()).1;
        let children: Vec<(bool, u32, f64, f64, f64, f64)> = (0..POOL)
            .into_par_iter()
            .map(|i| {
                let mut child = founder.clone();
                let mut rng = SmallRng::seed_from_u64(0xE3_C0_00_00 + pool * 1_000 + i as u64);
                let summary = MutationEngine::apply_mutations_with_food_type_count(
                    &mut child,
                    &setup.config.mutation,
                    &reachable,
                    ParentExecuted::Record(&frozen.record, frozen.age),
                    &mut rng,
                    food_types,
                );
                if child == founder {
                    return (true, summary.applied_events, 0.0, 0.0, 0.0, 0.0);
                }
                let a = mean(&scores(&setup, &child, &batch_a)) - founder_a;
                let b = mean(&scores(&setup, &child, &batch_b)) - founder_b;
                let bank_scores = scores(&setup, &child, &bank);
                let deltas: Vec<f64> = bank_scores
                    .iter()
                    .zip(&founder_bank)
                    .map(|(c, f)| c - f)
                    .collect();
                let wins = deltas.iter().filter(|d| **d > 0.0).count() as f64;
                let losses = deltas.iter().filter(|d| **d < 0.0).count() as f64;
                (
                    false,
                    summary.applied_events,
                    a,
                    b,
                    mean(&deltas),
                    wins - losses,
                )
            })
            .collect();
        // Truncation on batch A over the whole pool (identical children score 0).
        let mut order: Vec<usize> = (0..POOL).collect();
        order.sort_by(|x, y| {
            children[*y]
                .2
                .partial_cmp(&children[*x].2)
                .unwrap()
                .then(x.cmp(y))
        });
        let trunc: Vec<f64> = order[..KEEP].iter().map(|i| children[*i].4).collect();
        let random: Vec<f64> = (0..KEEP)
            .map(|k| children[(k * 4 + pool as usize) % POOL].4)
            .collect();
        summary_pools.push(
            json!({"pool": pool, "founder_a": founder_a, "founder_b": founder_b,
            "trunc_bank_gain": mean(&trunc), "random_bank_gain": mean(&random)}),
        );
        for (i, c) in children.iter().enumerate() {
            rows.push((pool, i, *c));
        }
    }
    let changed: Vec<&(u64, usize, (bool, u32, f64, f64, f64, f64))> =
        rows.iter().filter(|r| !r.2 .0).collect();
    let da: Vec<f64> = changed.iter().map(|r| r.2 .2).collect();
    let db: Vec<f64> = changed.iter().map(|r| r.2 .3).collect();
    let dbank: Vec<f64> = changed.iter().map(|r| r.2 .4).collect();
    let a_up = changed.iter().filter(|r| r.2 .2 > 0.0).count();
    let a_up_bank_up = changed
        .iter()
        .filter(|r| r.2 .2 > 0.0 && r.2 .4 > 0.0)
        .count();
    let bank_up = changed.iter().filter(|r| r.2 .4 > 0.0).count();
    let bank_up_sign = changed
        .iter()
        .filter(|r| r.2 .4 > 0.0 && r.2 .5 >= 4.0)
        .count();
    let bank_changed = changed.iter().filter(|r| r.2 .4 != 0.0).count();
    let mut hist: BTreeMap<String, usize> = BTreeMap::new();
    for d in &dbank {
        let k = if *d > 1.0 {
            ">+1"
        } else if *d > 0.0 {
            "(0,+1]"
        } else if *d == 0.0 {
            "0"
        } else if *d >= -1.0 {
            "[-1,0)"
        } else {
            "<-1"
        };
        *hist.entry(k.to_owned()).or_default() += 1;
    }
    let pools_trunc: Vec<f64> = summary_pools
        .iter()
        .map(|p| p["trunc_bank_gain"].as_f64().unwrap())
        .collect();
    let pools_rand: Vec<f64> = summary_pools
        .iter()
        .map(|p| p["random_bank_gain"].as_f64().unwrap())
        .collect();
    let fa: Vec<f64> = summary_pools
        .iter()
        .map(|p| p["founder_a"].as_f64().unwrap())
        .collect();
    println!(
        "{}",
        json!({
            "probe": "e3", "children": rows.len(), "identical": rows.len() - changed.len(),
            "changed_genome": changed.len(), "bank_score_changed": bank_changed,
            "founder_bank_mean": mean(&founder_bank), "founder_4scene_means": fa,
            "corr_a_bank": corr(&da, &dbank), "corr_a_b": corr(&da, &db),
            "a_improved": a_up, "a_improved_and_bank_improved": a_up_bank_up,
            "bank_improved": bank_up, "bank_improved_sign4": bank_up_sign, "bank_delta_hist": hist,
            "trunc_bank_gain_by_pool": pools_trunc, "random_bank_gain_by_pool": pools_rand,
            "trunc_minus_random_mean": mean(&pools_trunc) - mean(&pools_rand),
        })
    );
    if let Some(mut f) = out_file() {
        for (pool, i, c) in &rows {
            writeln!(
                f,
                "{}",
                json!({"pool": pool, "i": i, "identical": c.0, "applied": c.1,
                "da": c.2, "db": c.3, "dbank": c.4, "sign": c.5})
            )
            .unwrap();
        }
    }
}

/// E4, elite neighbourhood audit (H3, H4, H8). Parents: the founder and the
/// final elites of a stored campaign (`PETRI_E4_ELITES`, an `elites/`
/// directory). Each parent gets 96 children scored on the E3 bank against the
/// parent; improvers are tallied by applied operator and by whether the
/// event's target node is a founder node (ids 0 and 1) or one added later.
#[test]
#[ignore = "exploration probe; minutes in release"]
fn e4_elite_neighbourhood_audit() {
    const CHILDREN: u64 = 96;
    let setup = setup();
    let founder = founder_genome_with_age_gate(
        setup.config.population.founder_profile,
        &setup.config.energy.lifecycle,
    );
    let bank = scenes(0xE3_BA_4C, 32, &setup);
    let batch = scenes(0xE4_A0_00, 4, &setup);
    let food_types = setup.config.world.food.types.len();
    let dir = std::env::var("PETRI_E4_ELITES").expect("PETRI_E4_ELITES");
    let mut parents: Vec<(String, CreatureGenome)> = vec![("founder".into(), founder.clone())];
    for arm in ["native", "shuffled-score"] {
        for r in 0..8 {
            let path = format!("{dir}/{arm}-{r}.json");
            let file: v3_lab::GenomeFile =
                serde_json::from_str(&std::fs::read_to_string(&path).expect("elite"))
                    .expect("genome file");
            parents.push((format!("{arm}-{r}"), file.genome));
        }
    }
    let mut out = out_file();
    for (name, parent) in &parents {
        let reachable = mesh_reachable_nodes(parent);
        let parent_bank = scores(&setup, parent, &bank);
        let frozen: Frozen = evaluate_genome(&setup, parent, batch.last().unwrap()).1;
        let kids: Vec<(bool, f64, Vec<String>, Vec<bool>)> = (0..CHILDREN)
            .into_par_iter()
            .map(|i| {
                let mut child = parent.clone();
                let mut rng = SmallRng::seed_from_u64(0xE4_C0_00_00 + i);
                let summary = MutationEngine::apply_mutations_with_food_type_count(
                    &mut child,
                    &setup.config.mutation,
                    &reachable,
                    ParentExecuted::Record(&frozen.record, frozen.age),
                    &mut rng,
                    food_types,
                );
                let ops: Vec<String> = summary
                    .events
                    .iter()
                    .filter(|e| format!("{:?}", e.outcome).starts_with("Applied"))
                    .map(|e| e.operator.map_or("none".into(), |o| format!("{o:?}")))
                    .collect();
                let founder_target: Vec<bool> = summary
                    .events
                    .iter()
                    .filter(|e| format!("{:?}", e.outcome).starts_with("Applied"))
                    .map(|e| e.target.is_some_and(|t| t.0 <= 1))
                    .collect();
                if child == *parent {
                    return (true, 0.0, ops, founder_target);
                }
                let d: Vec<f64> = scores(&setup, &child, &bank)
                    .iter()
                    .zip(&parent_bank)
                    .map(|(c, p)| c - p)
                    .collect();
                (false, mean(&d), ops, founder_target)
            })
            .collect();
        let changed: Vec<_> = kids.iter().filter(|k| !k.0).collect();
        let improved: Vec<_> = changed.iter().filter(|k| k.1 > 0.0).collect();
        let mut ops_improved: BTreeMap<String, usize> = BTreeMap::new();
        for k in &improved {
            for o in &k.2 {
                *ops_improved.entry(o.clone()).or_default() += 1;
            }
        }
        let improved_founder_only = improved
            .iter()
            .filter(|k| !k.3.is_empty() && k.3.iter().all(|b| *b))
            .count();
        let improved_new_node = improved.iter().filter(|k| k.3.iter().any(|b| !b)).count();
        let row = json!({
            "probe": "e4", "parent": name, "nodes": parent.nodes.len(), "genome_size": parent.genome_size(),
            "parent_bank_mean": mean(&parent_bank), "children": CHILDREN, "changed": changed.len(),
            "bank_silent": changed.iter().filter(|k| k.1 == 0.0).count(),
            "improved": improved.len(), "worse": changed.iter().filter(|k| k.1 < 0.0).count(),
            "improved_gt1": improved.iter().filter(|k| k.1 > 1.0).count(),
            "max_gain": improved.iter().map(|k| k.1).fold(0.0_f64, f64::max),
            "improved_targets_founder_nodes_only": improved_founder_only,
            "improved_touch_added_node": improved_new_node,
            "ops_improved": ops_improved,
        });
        println!("{row}");
        if let Some(f) = out.as_mut() {
            writeln!(f, "{row}").unwrap();
        }
    }
}

fn elite(dir: &str, name: &str) -> CreatureGenome {
    let path = format!("{dir}/{name}.json");
    let file: v3_lab::GenomeFile =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("elite")).expect("genome file");
    file.genome
}

fn child_of(setup: &Setup, parent: &CreatureGenome, frozen: &Frozen, seed: u64) -> CreatureGenome {
    let mut child = parent.clone();
    let mut rng = SmallRng::seed_from_u64(seed);
    MutationEngine::apply_mutations_with_food_type_count(
        &mut child,
        &setup.config.mutation,
        &mesh_reachable_nodes(parent),
        ParentExecuted::Record(&frozen.record, frozen.age),
        &mut rng,
        setup.config.world.food.types.len(),
    );
    child
}

/// E7, two-step neighbourhood of the plateau (H3) plus the sterility check.
/// Parent: E1's `native-1` elite (`PETRI_E4_ELITES`). Up to 32 genome-changed,
/// bank-silent children (zero delta on every bank scene) each get 16
/// grandchildren; the matched control is as many further direct children.
/// Also prints founder and plateau bank totals of food, moves and penalty.
#[test]
#[ignore = "exploration probe; minutes in release"]
fn e7_two_step_and_sterility() {
    let setup = setup();
    let founder = founder_genome_with_age_gate(
        setup.config.population.founder_profile,
        &setup.config.energy.lifecycle,
    );
    let dir = std::env::var("PETRI_E4_ELITES").expect("PETRI_E4_ELITES");
    let plateau = elite(&dir, "native-1");
    let bank = scenes(0xE3_BA_4C, 32, &setup);
    let batch = scenes(0xE4_A0_00, 4, &setup);
    for (name, g) in [("founder", &founder), ("plateau", &plateau)] {
        let s: Vec<_> = bank
            .iter()
            .map(|sc| evaluate_genome(&setup, g, sc).0)
            .collect();
        println!(
            "{}",
            json!({"probe": "e7-sterility", "genome": name,
            "score": s.iter().map(|x| x.score).sum::<f64>() / 32.0,
            "food_eaten": s.iter().map(|x| x.food_eaten).sum::<u32>(),
            "moves_attempted": s.iter().map(|x| x.moves_attempted).sum::<u64>(),
            "penalty_charged": s.iter().map(|x| x.penalty_charged).sum::<f64>(),
            "deaths": s.iter().filter(|x| x.death_tick.is_some()).count()})
        );
    }
    let parent_bank = scores(&setup, &plateau, &bank);
    let frozen: Frozen = evaluate_genome(&setup, &plateau, batch.last().unwrap()).1;
    let delta = |g: &CreatureGenome| -> Vec<f64> {
        scores(&setup, g, &bank)
            .iter()
            .zip(&parent_bank)
            .map(|(c, p)| c - p)
            .collect()
    };
    let silent: Vec<CreatureGenome> = (0..256u64)
        .into_par_iter()
        .filter_map(|i| {
            let c = child_of(&setup, &plateau, &frozen, 0xE7_10_00_00 + i);
            (c != plateau && delta(&c).iter().all(|d| *d == 0.0)).then_some(c)
        })
        .collect();
    let silent: Vec<CreatureGenome> = silent.into_iter().take(32).collect();
    let tally = |gains: &[f64]| {
        json!({"n": gains.len(), "improved": gains.iter().filter(|g| **g > 0.0).count(),
        "improved_gt1": gains.iter().filter(|g| **g > 1.0).count(),
        "max": gains.iter().copied().fold(f64::MIN, f64::max)})
    };
    let mut grand = Vec::new();
    for (k, s) in silent.iter().enumerate() {
        // The silent child's own record on the same batch, as a parent's would be.
        let f = evaluate_genome(&setup, s, batch.last().unwrap()).1;
        let gains: Vec<f64> = (0..16u64)
            .into_par_iter()
            .map(|j| {
                mean(&delta(&child_of(
                    &setup,
                    s,
                    &f,
                    0xE7_20_00_00 + k as u64 * 100 + j,
                )))
            })
            .collect();
        grand.extend(gains);
    }
    let direct: Vec<f64> = (0..grand.len() as u64)
        .into_par_iter()
        .map(|i| {
            mean(&delta(&child_of(
                &setup,
                &plateau,
                &frozen,
                0xE7_30_00_00 + i,
            )))
        })
        .collect();
    println!(
        "{}",
        json!({"probe": "e7-two-step", "silent_children": silent.len(),
        "grandchildren": tally(&grand), "direct_children": tally(&direct)})
    );
}

// Run 2 (`docs/strategy/evolvability-exploration-2026-10-run2.md`). Diagnostic
// genomes (run 2 plan rule 7) are hand-edited, scored on fixed scenes only,
// and never start a lineage, enter selection or seed an arm.

fn graph_mut(genome: &mut CreatureGenome, node: usize) -> &mut CgpGraphBackendDef {
    match &mut genome.nodes[node].backend_def {
        BackendDef::Graph(def) => def,
        BackendDef::Vm(_) => panic!("node {node} is not a Graph node"),
    }
}

/// The readiness signal silenced: node 0's `CustomOutput(1)` (the founder's
/// can-reproduce slot) loses every input edge.
fn silenced(genome: &CreatureGenome) -> CreatureGenome {
    let mut g = genome.clone();
    graph_mut(&mut g, 0)
        .sink_mut(OutputSinkKind::CustomOutput(1))
        .expect("fixed catalog")
        .inputs
        .clear();
    g
}

/// Node 1's input reference `idx` re-pointed to upstream slot `slot`.
fn repointed(genome: &CreatureGenome, pairs: &[(usize, usize)]) -> CreatureGenome {
    let mut g = genome.clone();
    for &(idx, slot) in pairs {
        g.nodes[1].input_refs[idx] = InputReference::UpstreamSlot(slot);
    }
    g
}

/// The plateau with the readiness signal restored: node 0 regains the
/// founder's `Multiply(CN0, CN1)` feeding `CustomOutput(1)`, and node 1's
/// input ref 1 reads slot 1 again. Every other plateau edit is kept.
fn restored(plateau: &CreatureGenome) -> CreatureGenome {
    let mut g = plateau.clone();
    let def = graph_mut(&mut g, 0);
    let idx = u16::try_from(def.compute_nodes.len()).expect("small graph");
    def.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Multiply,
        inputs: vec![
            GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            },
            GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            },
        ],
        plasticity: None,
    });
    def.sink_mut(OutputSinkKind::CustomOutput(1))
        .expect("fixed catalog")
        .inputs = vec![GraphEdge {
        source: GraphSource::ComputeNode(idx),
        weight: 1.0,
    }];
    repointed(&g, &[(1, 1)])
}

/// Bank totals for one genome: mean score, food, moves, penalty, deaths.
fn bank_row(
    setup: &Setup,
    name: &str,
    genome: &CreatureGenome,
    bank: &[Scene],
) -> serde_json::Value {
    let s: Vec<_> = bank
        .par_iter()
        .map(|sc| evaluate_genome(setup, genome, sc).0)
        .collect();
    json!({"genome": name,
        "score": s.iter().map(|x| x.score).sum::<f64>() / s.len() as f64,
        "food_eaten": s.iter().map(|x| x.food_eaten).sum::<u32>(),
        "moves_attempted": s.iter().map(|x| x.moves_attempted).sum::<u64>(),
        "penalty_charged": s.iter().map(|x| x.penalty_charged).sum::<f64>(),
        "deaths": s.iter().filter(|x| x.death_tick.is_some()).count(),
        "scores": s.iter().map(|x| x.score).collect::<Vec<_>>()})
}

/// P1, sterility-shortcut diagnostics (run 2 phase 1) on run 1's E3 bank:
/// the founder 2 × 2 (readiness silenced × the plateau's upstream
/// re-pointing), its decomposition, the plateau and its restoration, and
/// three elites with an intact readiness path, silenced and unsilenced.
#[test]
#[ignore = "exploration probe; seconds in release"]
fn p1_sterility_diagnostics() {
    const MARGIN: f64 = 0.5;
    let setup = setup();
    let founder = founder_genome_with_age_gate(
        setup.config.population.founder_profile,
        &setup.config.energy.lifecycle,
    );
    let dir = std::env::var("PETRI_E4_ELITES").expect("PETRI_E4_ELITES");
    let plateau = elite(&dir, "native-1");
    let bank = scenes(0xE3_BA_4C, 32, &setup);
    let repoint = [(1, 12), (2, 9)];
    let mut genomes: Vec<(String, CreatureGenome)> = vec![
        ("founder".into(), founder.clone()),
        ("founder+silenced".into(), silenced(&founder)),
        ("founder+repointed".into(), repointed(&founder, &repoint)),
        (
            "founder+silenced+repointed".into(),
            silenced(&repointed(&founder, &repoint)),
        ),
        ("founder+ref1".into(), repointed(&founder, &[(1, 12)])),
        ("founder+ref2".into(), repointed(&founder, &[(2, 9)])),
        ("plateau".into(), plateau.clone()),
        ("plateau+restored".into(), restored(&plateau)),
    ];
    for name in ["native-0", "native-5", "shuffled-score-5"] {
        let g = elite(&dir, name);
        genomes.push((format!("{name}+silenced"), silenced(&g)));
        genomes.push((name.into(), g));
    }
    let mut out = out_file();
    let mut score = BTreeMap::new();
    for (name, g) in &genomes {
        let row = bank_row(&setup, name, g, &bank);
        score.insert(name.clone(), row["score"].as_f64().unwrap());
        let mut row = row;
        row["probe"] = json!("p1");
        println!("{row}");
        if let Some(f) = out.as_mut() {
            writeln!(f, "{row}").unwrap();
        }
    }
    let s = |k: &str| score[k];
    let gain = s("plateau") - s("founder");
    let silence_alone = s("founder+silenced") - s("founder");
    let silence_with_repoint = s("founder+silenced+repointed") - s("founder+repointed");
    let restoration = s("plateau") - s("plateau+restored");
    let mut contrasts = vec![
        ("founder", silence_alone),
        ("founder+repointed", silence_with_repoint),
        ("plateau-restored", restoration),
    ];
    for name in ["native-0", "native-5", "shuffled-score-5"] {
        contrasts.push((name, s(&format!("{name}+silenced")) - s(name)));
    }
    let dominant = silence_alone >= 0.7 * gain && restoration >= 0.7 * gain;
    let contributor = !dominant && contrasts.iter().any(|(_, d)| *d > MARGIN);
    let summary = json!({"probe": "p1-verdict", "margin": MARGIN, "gain": gain,
        "silence_main": (silence_alone + silence_with_repoint) / 2.0,
        "repoint_main": ((s("founder+repointed") - s("founder"))
            + (s("founder+silenced+repointed") - s("founder+silenced"))) / 2.0,
        "interaction": silence_with_repoint - silence_alone,
        "silence_alone_share": silence_alone / gain, "restoration_share": restoration / gain,
        "contrasts": contrasts.iter().map(|(n, d)| json!({"genome": n, "silencing_delta": d})).collect::<Vec<_>>(),
        "verdict": if dominant { "dominant" } else if contributor { "contributor" } else { "no-shortcut" }});
    println!("{summary}");
    if let Some(f) = out.as_mut() {
        writeln!(f, "{summary}").unwrap();
    }
}
