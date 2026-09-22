//! Benchmark profiles: schema constants, recipes, and simulation config builders.

use super::schema::ProfileBlock;
use crate::six;
use serde::{Deserialize, Serialize};
use v3_core::config::SimulationConfig;
use v3_core::neighborhood;

pub const SCHEMA_VERSION: u32 = 1;

/// The deterministic work counters normalized by creature-tick. The last
/// three, `pass_cap_hits` (T19.F02), `passes`, and `decided_passes`
/// (T19.F04), are additive: a stored report without one reads as unmeasured
/// (`level: "new"`) in comparisons and zero in its totals.
pub const COUNTER_NAMES: [&str; 9] = [
    "mesh_hops",
    "vm_steps",
    "graph_relax_iters",
    "plasticity_updates",
    "actions_applied",
    "births",
    "pass_cap_hits",
    "passes",
    "decided_passes",
];

/// Counters added after the first stored summaries: absent from an older
/// summary and read as zero when such a summary is the current side of an
/// offline comparison (as a reference, absence stays `level: "new"`).
pub const ADDITIVE_COUNTER_NAMES: [&str; 3] = ["pass_cap_hits", "passes", "decided_passes"];

/// Persistence sampling cadence (T01.F11): every executed tick that is a
/// multiple of this constant is sampled, plus the last executed tick.
pub const SAMPLE_EVERY_TICKS: u64 = 100;

/// Mutational-neighborhood (T11.F01) battery trial and birth counts. Carried
/// on `ProfileParams` rather than fixed constants, so a tiny test fixture can
/// exercise the same goal/gate observation code path as production at a
/// fraction of the cost (the T01.F12 precedent for the lineage and memory
/// indicators). Never written to `ProfileBlock` — `compare_against_path`
/// hard-fails on `ProfileBlock` inequality, and only `gate_profile_params()`
/// and `goal_profile_params()` carry the production reading the floors and
/// the no-regression rule depend on. The actual sizes used are recorded
/// truthfully in the report's `NeighborhoodBattery` block instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeighborhoodSizes {
    pub founder_operator_trials: u32,
    pub founder_births: u32,
    pub evolved_operator_trials: u32,
    pub evolved_births: u32,
    /// Neighborhood read (T14.F12): genomes sampled per world and production
    /// births per sampled genome.
    pub read_sample: u32,
    pub read_births: u32,
}

impl NeighborhoodSizes {
    /// The production sizes, halved once from the originally predeclared
    /// 100 operator trials / 1,000 births after a debug-build timing
    /// measurement showed the founder half's two full-report calls inside
    /// `gate_profile_deterministic_block_is_byte_identical_across_two_runs`
    /// growing that test well past its 10-second budget (see the T11.F01
    /// spec's Verification section for the measured numbers). Release wall
    /// time at the original sizes was ~0.8s against the 10s release budget,
    /// so the halving is driven by the debug-build constraint alone. The
    /// evolved-half sizes are unaffected by that halving, since the evolved
    /// half never runs inside a debug-build gate test.
    pub const PRODUCTION: Self = Self {
        founder_operator_trials: 50,
        founder_births: 500,
        evolved_operator_trials: 20,
        evolved_births: 200,
        read_sample: 50,
        read_births: 100,
    };
}

impl Default for NeighborhoodSizes {
    /// Small sizes for test fixtures and synthetic profiles: fast, and never
    /// mistaken for a production reading.
    fn default() -> Self {
        Self {
            founder_operator_trials: 2,
            founder_births: 5,
            evolved_operator_trials: 2,
            evolved_births: 5,
            read_sample: 3,
            read_births: 4,
        }
    }
}

// ── Profile parameters ──────────────────────────────────────────────────────

/// A complete resolved recipe and its supplied source path.
#[derive(Debug, Clone)]
pub struct Recipe {
    pub path: String,
    pub config: SimulationConfig,
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && serde_json::to_value(&self.config).expect("config serializes")
                == serde_json::to_value(&other.config).expect("config serializes")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileParams {
    pub recipe: Option<Recipe>,
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub founders: u32,
    pub seeds: Vec<u64>,
    pub ticks: u64,
    /// `None` leaves `SimulationConfig::default()`'s per-food-type coverage
    /// untouched (production defaults) and serializes as the profile string
    /// `default`; `Some(x)` forces `x` onto every food type.
    pub food_coverage: Option<f32>,
    pub neighborhood: NeighborhoodSizes,
    pub drift: neighborhood::drift::DriftSizes,
    /// Sizes of the goal-only recruitment-paths experiment. Production
    /// profiles run `Sizes::PRODUCTION`; test fixtures pass `Sizes::TEST`,
    /// which `cfg(test)` cannot select for integration tests.
    pub recruitment: neighborhood::recruitment_paths::Sizes,
}

/// Predeclared gate profile constants (T10.F10 Inputs and Invariants).
///
/// The horizon was reduced from the originally predeclared 300 ticks to 75
/// ticks after a timing probe on the recording host measured the two fast
/// `make check` tests (nine seed-runs total, debug build) would otherwise
/// take roughly 75 seconds, well over the 30-second budget. World size,
/// founder count, seeds, and food coverage are unchanged from the
/// predeclaration.
pub fn gate_profile_params() -> ProfileParams {
    ProfileParams {
        recipe: None,
        name: "gate".to_string(),
        width: 128,
        height: 128,
        founders: 256,
        seeds: vec![11, 22, 33],
        ticks: 75,
        food_coverage: Some(1.0),
        neighborhood: NeighborhoodSizes::PRODUCTION,
        drift: neighborhood::drift::DriftSizes::PRODUCTION,
        recruitment: neighborhood::recruitment_paths::Sizes::PRODUCTION,
    }
}

/// Predeclared minutes-scale goal profile constants (T01.F12).
pub fn goal_profile_params() -> ProfileParams {
    ProfileParams {
        recipe: None,
        name: GOAL_WORLD_SET.to_string(),
        width: 1600,
        height: 1600,
        founders: 10_000,
        seeds: goal_recipe_seeds(),
        ticks: 2_000,
        food_coverage: None,
        neighborhood: NeighborhoodSizes::PRODUCTION,
        drift: neighborhood::drift::DriftSizes::PRODUCTION,
        recruitment: neighborhood::recruitment_paths::Sizes::PRODUCTION,
    }
}

pub const GOAL_WORLD_SET: &str = "goal-worlds-v1";

/// One checked-in baseline world: the recipe the world-set profile runs and
/// the seed it runs it at. Adding a world to the set is one entry here — the
/// profile's seed list is derived from these seeds, and a profile that names
/// any other list is rejected rather than silently mis-paired.
pub(super) struct GoalRecipe {
    name: &'static str,
    path: &'static str,
    source: &'static str,
    seed: u64,
}

pub(super) const GOAL_RECIPES: [GoalRecipe; 3] = [
    GoalRecipe {
        name: "Orchards in grassland",
        path: "experiments/worlds/orchards-in-grassland.json",
        source: include_str!("../../../../experiments/worlds/orchards-in-grassland.json"),
        seed: 11,
    },
    GoalRecipe {
        name: "Canyon country",
        path: "experiments/worlds/canyon-country.json",
        source: include_str!("../../../../experiments/worlds/canyon-country.json"),
        seed: 22,
    },
    GoalRecipe {
        name: "Confluence",
        path: "experiments/worlds/confluence.json",
        source: include_str!("../../../../experiments/worlds/confluence.json"),
        seed: 33,
    },
];

/// The seeds the checked-in world set runs, in report order.
pub(super) fn goal_recipe_seeds() -> Vec<u64> {
    GOAL_RECIPES.iter().map(|recipe| recipe.seed).collect()
}

/// The recipe each of a profile's seeds runs: the checked-in world set for
/// [`GOAL_WORLD_SET`], and nothing at all for every other profile, whose seeds
/// are replicates of one config.
///
/// # Errors
///
/// When a world-set profile's seed list is not exactly the seeds its recipes
/// carry, in their order. Pairing by position would then run a world under
/// another world's recipe, drop a world the set gained, or index past the
/// recipe list, so this is a hard error before any run starts.
pub(super) fn goal_recipes_for(params: &ProfileParams) -> Result<&'static [GoalRecipe], String> {
    if params.name != GOAL_WORLD_SET {
        return Ok(&[]);
    }
    let seeds = goal_recipe_seeds();
    if params.seeds != seeds {
        return Err(format!(
            "profile {GOAL_WORLD_SET} runs one checked-in recipe per seed and must name their \
             seeds in order {seeds:?}, not {:?}",
            params.seeds
        ));
    }
    Ok(&GOAL_RECIPES)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalCase {
    pub name: String,
    pub seed: u64,
    pub recipe_path: String,
    pub config_digest: String,
    pub food_type_count: usize,
}

pub(super) fn goal_case(
    params: &ProfileParams,
    recipe: &GoalRecipe,
) -> (GoalCase, SimulationConfig) {
    let config = v3_core::config::resolve_config(
        &SimulationConfig::default(),
        serde_json::from_str(recipe.source).expect("checked-in recipe JSON"),
    )
    .expect("checked-in goal recipe");
    let mut case_params = params.clone();
    case_params.recipe = Some(Recipe {
        path: recipe.path.to_string(),
        config,
    });
    let config = build_config(&case_params);
    (
        GoalCase {
            name: recipe.name.to_string(),
            seed: recipe.seed,
            recipe_path: recipe.path.to_string(),
            config_digest: v3_core::config::config_digest(&config),
            food_type_count: config.world.food.types.len(),
        },
        config,
    )
}

/// The `profile.food_coverage` report string for a profile that leaves
/// production food coverage untouched.
const DEFAULT_FOOD_COVERAGE: &str = "default";

pub fn build_config(params: &ProfileParams) -> SimulationConfig {
    let mut config = params
        .recipe
        .as_ref()
        .map_or_else(SimulationConfig::default, |recipe| recipe.config.clone());
    config.world.width = params.width;
    config.world.height = params.height;
    config.population.initial_creatures = params.founders;
    if let Some(coverage) = params.food_coverage {
        config.world.food.shared.initial_coverage = coverage;
        for food_type in &mut config.world.food.types {
            food_type.initial_coverage = coverage;
        }
    }
    config.normalize();
    if params.recipe.is_some() {
        config.apply_startup_overrides();
    }
    config
}

pub(super) fn profile_block(
    params: &ProfileParams,
    config: &SimulationConfig,
    recipes: &[GoalRecipe],
) -> ProfileBlock {
    ProfileBlock {
        cases: recipes
            .iter()
            .map(|recipe| goal_case(params, recipe).0)
            .collect(),
        recipe_path: params.recipe.as_ref().map(|recipe| recipe.path.clone()),
        config_digest: params
            .recipe
            .as_ref()
            .map(|_| v3_core::config::config_digest(config)),
        name: params.name.clone(),
        world_width: config.world.width,
        world_height: config.world.height,
        founders: config.population.initial_creatures,
        seeds: params.seeds.clone(),
        ticks: params.ticks,
        food_coverage: params.food_coverage.map_or_else(
            || {
                if params.name == GOAL_WORLD_SET {
                    "per-case recipe".to_string()
                } else if params.recipe.is_some() {
                    "recipe".to_string()
                } else {
                    DEFAULT_FOOD_COVERAGE.to_string()
                }
            },
            |coverage| {
                six(f64::from(if params.recipe.is_some() {
                    config.world.food.types[0].initial_coverage
                } else {
                    coverage
                }))
            },
        ),
    }
}

// ── Report schema ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
