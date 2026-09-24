use super::*;
use crate::bench::run::run_deterministic;
use crate::bench::tests::{small_profile, small_world_set_params};
use v3_core::simulation::seed_simulation;

#[test]
fn omitting_food_coverage_leaves_production_coverage_untouched() {
    let params = ProfileParams {
        recipe: None,
        name: "sweep".to_string(),
        width: 32,
        height: 32,
        founders: 8,
        seeds: vec![1],
        ticks: 10,
        food_coverage: None,
        neighborhood: NeighborhoodSizes::default(),
        drift: Default::default(),
        recruitment: neighborhood::recruitment_paths::Sizes::TEST,
        mutation_effects: Default::default(),
    };
    let config = build_config(&params);

    let default_config = SimulationConfig::default();
    let coverages: Vec<f32> = config
        .world
        .food
        .types
        .iter()
        .map(|t| t.initial_coverage)
        .collect();
    let default_coverages: Vec<f32> = default_config
        .world
        .food
        .types
        .iter()
        .map(|t| t.initial_coverage)
        .collect();

    assert_eq!(coverages, default_coverages);
    assert_eq!(coverages, vec![0.54]);
}

#[test]
fn forcing_food_coverage_applies_it_to_every_food_type() {
    let config = build_config(&gate_profile_params());

    for food_type in &config.world.food.types {
        assert!((food_type.initial_coverage - 1.0).abs() < 1e-6);
    }
}

#[test]
fn recipe_profile_identifies_final_config_without_fabricating_legacy_metadata() {
    let mut params = small_profile("sweep");
    let legacy = serde_json::to_value(profile_block(&params, &build_config(&params), &[])).unwrap();
    assert!(legacy.get("recipe_path").is_none());
    assert!(legacy.get("config_digest").is_none());
    let old: ProfileBlock = serde_json::from_value(legacy).unwrap();
    assert!(old.config_digest.is_none());
    let config = v3_core::config::resolve_config(&SimulationConfig::default(), serde_json::json!({"world":{"world_seed":18446744073709551615u64},"energy":{"costs":{"move_cost":0.75}}})).unwrap();
    params.recipe = Some(Recipe {
        path: "world.json".into(),
        config,
    });
    params.food_coverage = None;
    let applied = build_config(&params);
    let profile = profile_block(&params, &applied, &[]);
    assert_eq!(profile.recipe_path.as_deref(), Some("world.json"));
    assert_eq!(
        profile.config_digest,
        Some(v3_core::config::config_digest(&applied))
    );
    assert_eq!(profile.food_coverage, "recipe");
    assert_eq!(profile.world_width, applied.world.width);
    assert_eq!(profile.founders, applied.population.initial_creatures);
    let sim = seed_simulation(applied.clone(), 1);
    assert_eq!(sim.config.world.world_seed, Some(u64::MAX));
    assert_eq!(sim.config.energy.costs.move_cost, 0.75);
    let (observed, _) = run_deterministic(&params).expect("a valid profile");
    assert_eq!(observed.profile, profile);
    params
        .recipe
        .as_mut()
        .unwrap()
        .config
        .energy
        .costs
        .move_cost = 0.5;
    assert_ne!(profile_block(&params, &build_config(&params), &[]), profile);
    params.food_coverage = Some(2.0);
    assert_eq!(
        profile_block(&params, &build_config(&params), &[]).food_coverage,
        "1.000000"
    );
}
#[test]
fn recipe_equality_requires_both_source_path_and_complete_config() {
    let recipe = Recipe {
        path: "world.json".into(),
        config: SimulationConfig::default(),
    };
    assert_eq!(recipe, recipe.clone());
    let mut changed = recipe.clone();
    changed.path = "other.json".into();
    assert_ne!(recipe, changed);
    changed = recipe.clone();
    changed.config.world.world_seed = Some(u64::MAX);
    assert_ne!(recipe, changed);
}

/// The world set runs exactly the seeds its recipes carry, in their order,
/// so adding a world is one entry in `GOAL_RECIPES`. A profile that names
/// other seeds, or the same seeds in another order, is a hard error rather
/// than a world silently paired with another world's recipe or dropped.
#[test]
fn a_world_set_profile_must_name_the_seeds_its_recipes_carry() {
    assert_eq!(
        goal_profile_params().seeds,
        vec![11, 22, 33],
        "the checked-in case order the stored reports were recorded in"
    );

    // A test-sized world set, so a guard that fails to fire finishes and
    // fails an assertion instead of running the real 1600² profile.
    let mut reordered = small_world_set_params();
    reordered.seeds = vec![22, 11, 33];
    let error = run_deterministic(&reordered)
        .err()
        .expect("a reordered seed list pairs each world with another world's recipe");
    assert!(error.contains(GOAL_WORLD_SET), "{error}");
    assert!(error.contains("22, 11, 33"), "{error}");
    assert!(error.contains("11, 22, 33"), "{error}");

    let mut short = small_world_set_params();
    short.seeds.pop();
    assert!(
        run_deterministic(&short).is_err(),
        "a missing seed drops a world instead of running it"
    );

    let mut other = small_profile("sweep");
    other.seeds = vec![7, 9];
    assert!(
        run_deterministic(&other).is_ok(),
        "only the world set is tied to the checked-in recipes"
    );
}
