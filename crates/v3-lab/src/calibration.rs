//! The calibration gate: on the grid axis × lifetimes, score the founder,
//! the comparator and the three scripted instruments on the calibration
//! scenes; select the first passing point in selection order — the lowest
//! food fraction or the largest scale, then the shortest lifetime — that
//! also passes competence on the validation scenes.

use rayon::prelude::*;
use v3_core::config::SimulationConfig;
use v3_core::creature::genome::CreatureGenome;

use crate::eval::{evaluate_genome, evaluate_scripted, SceneScore, Scoring, Scripted, Setup};
use crate::rng::{hash, stream, Part};
use crate::scene::{Assay, Geometry, Scene, SceneSpec};
use crate::summary::{
    Calibration, CalibrationPoint, PointMeans, Selected, ThresholdSource, Verdict,
};

/// Minimum mean-score gap between successive sensitivity instruments.
pub const SENSITIVITY_GAP: f64 = 0.1;

/// The gate's inputs.
#[derive(Debug, Clone)]
pub struct GridInput {
    /// The arena's axis points, in any order.
    pub points: Vec<Geometry>,
    pub lifetimes: Vec<u32>,
    pub assay: Assay,
    pub scoring: Scoring,
    pub scenes: u32,
    pub validation_scenes: u32,
    pub margin: f64,
    pub calibration_seed: u64,
    pub validation_seed: u64,
    pub start_energy: f32,
    /// Explicit override of the reach threshold.
    pub reach_threshold: Option<f64>,
}

/// The gate's result.
#[derive(Debug, Clone)]
pub struct Gate {
    pub calibration: Calibration,
    /// Validation scenes at the selected point (empty when uncalibrated).
    pub validation: Vec<Scene>,
    /// The selected point's scene draw (none when uncalibrated).
    pub spec: Option<SceneSpec>,
    pub creature_ticks: u64,
}

/// `n` scenes from a fresh `seed` stream, or the per-scene redraw record up
/// to the infeasible draw.
///
/// # Errors
///
/// The redraw record when a draw is infeasible.
pub fn draw_scenes(seed: u64, n: u32, spec: &SceneSpec) -> Result<Vec<Scene>, Vec<u32>> {
    let mut rng = stream(&[Part::U(seed)]);
    let mut scenes = Vec::new();
    for _ in 0..n {
        match spec.draw(&mut rng) {
            Ok(scene) => scenes.push(scene),
            Err(infeasible) => {
                let mut redraws: Vec<u32> = scenes.iter().map(|s| s.redraws).collect();
                redraws.push(infeasible.redraws);
                return Err(redraws);
            }
        }
    }
    Ok(scenes)
}

/// Scripted actor seed `hash(parent, "scripted", arm, scene)`.
#[must_use]
pub fn scripted_seed(parent: u64, policy: Scripted, scene: usize) -> u64 {
    hash(&[
        Part::U(parent),
        Part::S("scripted"),
        Part::S(policy.name()),
        Part::U(scene as u64),
    ])
}

struct Scored {
    means: PointMeans,
    creature_ticks: u64,
}

/// Per-scene actor order in [`score_point`].
const FOUNDER: usize = 0;
const COMPARATOR: usize = 1;
const FLOOR: usize = 2;
const HALF: usize = 3;
const ORACLE: usize = 4;

/// Mean over scenes of `field` of actor `actor`.
fn mean_of(rows: &[[SceneScore; 5]], actor: usize, field: impl Fn(&SceneScore) -> f64) -> f64 {
    let values: Vec<f64> = rows.iter().map(|row| field(&row[actor])).collect();
    crate::stats::mean(&values).unwrap_or(0.0)
}

fn score_point(
    setup: &Setup,
    founder: &CreatureGenome,
    comparator: &CreatureGenome,
    scenes: &[Scene],
    scripted_parent: u64,
) -> Scored {
    let per_scene: Vec<[SceneScore; 5]> = scenes
        .par_iter()
        .enumerate()
        .map(|(index, scene)| {
            let scripted = |policy| {
                evaluate_scripted(
                    setup,
                    founder,
                    policy,
                    scene,
                    scripted_seed(scripted_parent, policy, index),
                )
            };
            [
                evaluate_genome(setup, founder, scene).0,
                evaluate_genome(setup, comparator, scene).0,
                scripted(Scripted::RandomWalk),
                scripted(Scripted::HalfSeeker),
                scripted(Scripted::OracleSeeker),
            ]
        })
        .collect();
    let wins = per_scene
        .iter()
        .filter(|row| row[COMPARATOR].score > row[FLOOR].score)
        .count();
    let creature_ticks = per_scene
        .iter()
        .map(|row| u64::from(row[FOUNDER].ticks) + u64::from(row[COMPARATOR].ticks))
        .sum();
    let score = |s: &SceneScore| s.score;
    let progress = |s: &SceneScore| s.progress;
    let blocked = SceneScore::blocked_fraction;
    let efficiency = |s: &SceneScore| s.efficiency;
    Scored {
        means: PointMeans {
            founder: mean_of(&per_scene, FOUNDER, score),
            floor: mean_of(&per_scene, FLOOR, score),
            half: mean_of(&per_scene, HALF, score),
            oracle: mean_of(&per_scene, ORACLE, score),
            comparator: mean_of(&per_scene, COMPARATOR, score),
            floor_progress: mean_of(&per_scene, FLOOR, progress),
            comparator_progress: mean_of(&per_scene, COMPARATOR, progress),
            floor_blocked_fraction: mean_of(&per_scene, FLOOR, blocked),
            comparator_blocked_fraction: mean_of(&per_scene, COMPARATOR, blocked),
            floor_efficiency: mean_of(&per_scene, FLOOR, efficiency),
            half_efficiency: mean_of(&per_scene, HALF, efficiency),
            oracle_efficiency: mean_of(&per_scene, ORACLE, efficiency),
            comparator_efficiency: mean_of(&per_scene, COMPARATOR, efficiency),
            comparator_wins: u32::try_from(wins).unwrap_or(u32::MAX),
        },
        creature_ticks,
    }
}

/// Comparator − floor ≥ `margin`, and the comparator above the floor on at
/// least `ceil(0.75 × n)` scenes.
#[must_use]
pub fn competent(means: &PointMeans, scenes: u32, margin: f64) -> bool {
    let needed = (3 * scenes).div_ceil(4);
    means.comparator - means.floor >= margin && means.comparator_wins >= needed
}

/// Floor < half < oracle with gaps ≥ 0.1, and graded comparator progress.
#[must_use]
pub fn sensitive(means: &PointMeans) -> bool {
    means.half - means.floor >= SENSITIVITY_GAP
        && means.oracle - means.half >= SENSITIVITY_GAP
        && means.comparator_progress > means.floor_progress
}

/// Selection order: food fractions ascending, scales descending (the
/// hardest calibrated detour); duplicates dropped.
fn selection_order(points: &[Geometry]) -> Vec<Geometry> {
    let mut points = points.to_vec();
    points.sort_by(|a, b| match (a.axis(), b.axis()) {
        ((Some(a), _), (Some(b), _)) => a.total_cmp(&b),
        ((_, Some(a)), (_, Some(b))) => b.cmp(&a),
        _ => std::cmp::Ordering::Equal,
    });
    points.dedup();
    points
}

/// Run the gate. `reference` is the resolved reference arm config.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn run_gate(
    reference: &SimulationConfig,
    founder: &CreatureGenome,
    comparator: &CreatureGenome,
    input: &GridInput,
) -> Gate {
    let spec = |geometry: &Geometry| SceneSpec {
        geometry: geometry.clone(),
        size: reference.world.width,
        vision_radius: reference.runtime.perception.vision_radius,
        assay: input.assay,
    };
    let setup = |lifetime| {
        Setup::new(reference.clone(), input.start_energy, lifetime).with_scoring(input.scoring)
    };
    let mut lifetimes = input.lifetimes.clone();
    lifetimes.sort_unstable();
    lifetimes.dedup();

    let mut creature_ticks = 0;
    let mut points = Vec::new();
    let geometries = selection_order(&input.points);
    for geometry in &geometries {
        let drawn = draw_scenes(input.calibration_seed, input.scenes, &spec(geometry));
        let (food_fraction, scale) = geometry.axis();
        for &lifetime in &lifetimes {
            let mut point = CalibrationPoint {
                food_fraction,
                scale,
                lifetime,
                scenes: input.scenes,
                redraws: Vec::new(),
                exposure: false,
                means: None,
                competence: false,
                sensitivity: false,
                validation_competence: None,
                validation: None,
            };
            match &drawn {
                Err(redraws) => point.redraws.clone_from(redraws),
                Ok(scenes) => {
                    point.redraws = scenes.iter().map(|s| s.redraws).collect();
                    point.exposure = true;
                    let scored = score_point(
                        &setup(lifetime),
                        founder,
                        comparator,
                        scenes,
                        input.calibration_seed,
                    );
                    creature_ticks += scored.creature_ticks;
                    point.competence = competent(&scored.means, input.scenes, input.margin);
                    point.sensitivity = sensitive(&scored.means);
                    point.means = Some(scored.means);
                }
            }
            points.push((point, geometry));
        }
    }

    let mut selected = None;
    let mut validation_scenes = Vec::new();
    let mut selected_spec = None;
    for (point, geometry) in &mut points {
        if !(point.exposure && point.competence && point.sensitivity) {
            continue;
        }
        let Ok(scenes) = draw_scenes(
            input.validation_seed,
            input.validation_scenes,
            &spec(geometry),
        ) else {
            point.validation_competence = Some(false);
            continue;
        };
        let scored = score_point(
            &setup(point.lifetime),
            founder,
            comparator,
            &scenes,
            input.validation_seed,
        );
        creature_ticks += scored.creature_ticks;
        let passed = competent(&scored.means, input.validation_scenes, input.margin);
        point.validation_competence = Some(passed);
        // Floor + 0.5 × (comparator − floor) on the validation means.
        let calibrated = scored.means.floor + 0.5 * (scored.means.comparator - scored.means.floor);
        point.validation = Some(scored.means.clone());
        if passed {
            let chosen = Selected {
                food_fraction: point.food_fraction,
                scale: point.scale,
                lifetime: point.lifetime,
                means: scored.means,
            };
            selected = Some((chosen, calibrated));
            validation_scenes = scenes;
            selected_spec = Some(spec(geometry));
            break;
        }
    }

    let (verdict, reach_threshold, source, selected) = match (selected, input.reach_threshold) {
        (None, _) => (Verdict::Uncalibrated, None, None, None),
        (Some((chosen, _)), Some(threshold)) => (
            Verdict::Calibrated,
            Some(threshold),
            Some(ThresholdSource::Override),
            Some(chosen),
        ),
        (Some((chosen, calibrated)), None) => (
            Verdict::Calibrated,
            Some(calibrated),
            Some(ThresholdSource::Calibrated),
            Some(chosen),
        ),
    };
    Gate {
        calibration: Calibration {
            margin: input.margin,
            points: points.into_iter().map(|(point, _)| point).collect(),
            selected,
            verdict,
            reach_threshold,
            reach_threshold_source: source,
        },
        validation: validation_scenes,
        spec: selected_spec,
        creature_ticks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn means(floor: f64, half: f64, oracle: f64, comparator: f64, wins: u32) -> PointMeans {
        PointMeans {
            founder: 0.0,
            floor,
            half,
            oracle,
            comparator,
            floor_progress: 0.1,
            comparator_progress: 0.2,
            floor_blocked_fraction: 0.0,
            comparator_blocked_fraction: 0.0,
            floor_efficiency: 0.0,
            half_efficiency: 0.0,
            oracle_efficiency: 0.0,
            comparator_efficiency: 0.0,
            comparator_wins: wins,
        }
    }

    use crate::arena::arena_config;
    use v3_core::creature::founder::founder_genome_with_age_gate;
    use v3_core::neighborhood::opportunity::controllers::controller;
    use v3_core::neighborhood::opportunity::Family;

    fn actors(config: &SimulationConfig) -> (CreatureGenome, CreatureGenome) {
        let founder = founder_genome_with_age_gate(
            config.population.founder_profile,
            &config.energy.lifecycle,
        );
        let comparator = controller(&founder, Family::Vector, false).0;
        (founder, comparator)
    }

    #[test]
    fn scripted_seeds_separate_parent_policy_and_scene() {
        let base = scripted_seed(7, Scripted::RandomWalk, 0);
        for other in [
            scripted_seed(8, Scripted::RandomWalk, 0),
            scripted_seed(7, Scripted::HalfSeeker, 0),
            scripted_seed(7, Scripted::OracleSeeker, 0),
            scripted_seed(7, Scripted::RandomWalk, 1),
        ] {
            assert_ne!(base, other);
        }
        assert!(base > 1);
    }

    #[test]
    fn foodless_scenes_score_no_wins_and_count_both_genomes_ticks() {
        let config = arena_config(48);
        let (founder, comparator) = actors(&config);
        let setup = Setup::new(config, 100.0, 5);
        let empty = |seed| Scene::open(seed, 48, Vec::new());
        let scored = score_point(&setup, &founder, &comparator, &[empty(1), empty(2)], 3);
        // Every actor scores 0: a comparator tie with the floor is no win.
        assert_eq!(scored.means.comparator, scored.means.floor);
        assert_eq!(scored.means.comparator_wins, 0);
        assert_eq!(scored.creature_ticks, 2 * 2 * 5);
    }

    fn input(margin: f64) -> GridInput {
        GridInput {
            points: vec![Geometry::SparseFood { fraction: 0.08 }],
            lifetimes: vec![100],
            assay: Assay::FoodSeeking,
            scoring: Scoring::BITES_AND_PROGRESS,
            scenes: 4,
            validation_scenes: 4,
            margin,
            calibration_seed: 11,
            validation_seed: 12,
            start_energy: 100.0,
            reach_threshold: None,
        }
    }

    fn genome_ticks(
        setup: &Setup,
        genomes: [&CreatureGenome; 2],
        seed: u64,
        n: u32,
        fraction: f64,
    ) -> u64 {
        let size = setup.config.world.width;
        let vision = setup.config.runtime.perception.vision_radius;
        draw_scenes(seed, n, &SceneSpec::sparse(size, fraction, vision))
            .unwrap()
            .iter()
            .flat_map(|scene| {
                genomes.map(|genome| u64::from(evaluate_genome(setup, genome, scene).0.ticks))
            })
            .sum()
    }

    #[test]
    fn selection_takes_the_lowest_fraction_or_the_largest_scale_first() {
        let fractions = [0.08, 0.02, 0.04, 0.02].map(|fraction| Geometry::SparseFood { fraction });
        assert_eq!(
            selection_order(&fractions),
            [0.02, 0.04, 0.08].map(|fraction| Geometry::SparseFood { fraction })
        );
        let scales = [1, 3, 2, 3].map(|scale| Geometry::Ring { scale });
        assert_eq!(
            selection_order(&scales),
            [3, 2, 1].map(|scale| Geometry::Ring { scale })
        );
    }

    #[test]
    fn a_calibrated_gate_counts_every_genome_tick_and_sets_the_midpoint_threshold() {
        let config = arena_config(48);
        let (founder, comparator) = actors(&config);
        let input = input(1.0);
        let gate = run_gate(&config, &founder, &comparator, &input);
        let selected = gate
            .calibration
            .selected
            .as_ref()
            .expect("tiny grid passes");
        let point = &gate.calibration.points[0];
        let v = point.validation.as_ref().expect("validated");
        assert_eq!(
            gate.calibration.reach_threshold,
            Some(v.floor + 0.5 * (v.comparator - v.floor))
        );
        let setup = Setup::new(config.clone(), input.start_energy, selected.lifetime);
        let both = [&founder, &comparator];
        let expected =
            genome_ticks(&setup, both, 11, 4, 0.08) + genome_ticks(&setup, both, 12, 4, 0.08);
        assert_eq!(gate.creature_ticks, expected);
    }

    #[test]
    fn points_failing_competence_are_never_validated() {
        let config = arena_config(48);
        let (founder, comparator) = actors(&config);
        let gate = run_gate(&config, &founder, &comparator, &input(1e9));
        let point = &gate.calibration.points[0];
        assert!(point.exposure && point.sensitivity && !point.competence);
        assert_eq!(point.validation_competence, None);
        assert_eq!(point.validation, None);
        assert!(gate.validation.is_empty());
    }

    #[test]
    fn competence_needs_the_margin_and_three_quarters_of_scenes() {
        assert!(competent(&means(0.5, 1.0, 2.0, 1.5, 12), 16, 1.0));
        assert!(!competent(&means(0.5, 1.0, 2.0, 1.4, 12), 16, 1.0));
        assert!(!competent(&means(0.5, 1.0, 2.0, 1.5, 11), 16, 1.0));
        // ceil(0.75 × 8) = 6 on the validation scenes; ceil(0.75 × 5) = 4.
        assert!(competent(&means(0.5, 1.0, 2.0, 1.5, 6), 8, 1.0));
        assert!(!competent(&means(0.5, 1.0, 2.0, 1.5, 3), 5, 1.0));
    }

    #[test]
    fn sensitivity_needs_ordered_gaps_and_graded_progress() {
        assert!(sensitive(&means(0.5, 0.75, 1.0, 1.0, 0)));
        assert!(!sensitive(&means(0.5, 0.55, 1.0, 1.0, 0)));
        assert!(!sensitive(&means(0.5, 0.75, 0.8, 1.0, 0)));
        let mut flat = means(0.5, 0.75, 1.0, 1.0, 0);
        flat.comparator_progress = flat.floor_progress;
        assert!(!sensitive(&flat));
    }
}
