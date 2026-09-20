use crate::contracts::OrdinaryFoodTypeId;

/// Edge mode for the world grid.
/// Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum WorldEdgeMode {
    #[default]
    Wrap,
    Bounded,
}

/// Shared ordinary-food substrate config. Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodResourceConfig {
    pub growth_rate: f32,
    pub initial_density: f32,
    pub initial_coverage: f32,
    pub spread_threshold_ratio: f32,
    pub spread_density_ratio: f32,
    pub recovery_spawn_rate: f32,
    pub recovery_floor_ratio: f32,
    pub max_density: f32,
    #[serde(default)]
    pub occupancy_depletion: OccupancyDepletionConfig,
    #[serde(default)]
    pub grazing: GrazingConfig,
    #[serde(default)]
    pub fertility: FertilityConfig,
    #[serde(default)]
    pub annealing: AnnealingConfig,
}

impl Default for FoodResourceConfig {
    fn default() -> Self {
        Self {
            growth_rate: 0.09,
            initial_density: 1.0,
            initial_coverage: 0.54,
            spread_threshold_ratio: 0.8,
            spread_density_ratio: 0.25,
            recovery_spawn_rate: 0.01,
            recovery_floor_ratio: 0.01,
            max_density: 1.0,
            occupancy_depletion: OccupancyDepletionConfig::default(),
            grazing: GrazingConfig::default(),
            fertility: FertilityConfig::default(),
            annealing: AnnealingConfig::default(),
        }
    }
}

/// Metadata for one configured ordinary-food type.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodTypeConfig {
    pub name: String,
    pub color: String,
    pub initial_density: f32,
    pub initial_coverage: f32,
    #[serde(default = "default_growth_inhibitor")]
    pub growth_inhibitor: f32,
    /// Energy per consumed density unit; None inherits the shared Eat reward.
    #[serde(default)]
    pub energy_per_unit: Option<f32>,
    /// Per-tick growth; None inherits shared growth_rate.
    #[serde(default)]
    pub growth_rate: Option<f32>,
    /// Recovery attempts per world cell per tick; None inherits the shared rate.
    #[serde(default)]
    pub recovery_spawn_rate: Option<f32>,
    /// Seed only passable cells with positive effective tick-zero fertility.
    #[serde(default)]
    pub initial_fertility_only: bool,
}

impl Default for FoodTypeConfig {
    fn default() -> Self {
        Self {
            name: "Primary Food".to_string(),
            color: "#22c55e".to_string(),
            initial_density: 1.0,
            initial_coverage: 0.54,
            growth_inhibitor: default_growth_inhibitor(),
            energy_per_unit: None,
            growth_rate: None,
            recovery_spawn_rate: None,
            initial_fertility_only: false,
        }
    }
}

const fn default_growth_inhibitor() -> f32 {
    0.2
}

/// Occupancy depletion configuration governing occupancy-driven food suppression.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OccupancyDepletionConfig {
    /// Whether occupancy depletion is applied during food growth.
    pub enabled: bool,
    /// Depletion deposited into the occupied cell each tick.
    pub deposit_per_occupied_tick: f32,
}

impl Default for OccupancyDepletionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            deposit_per_occupied_tick: 0.08,
        }
    }
}

/// Grazing recovery and overuse (T02.F04): a per-food-type, per-cell fertility
/// modifier in `[floor, 1.0]` that each consuming bite multiplies by `factor`
/// and that recovers linearly toward 1.0 by `1 / recovery_ticks` per tick.
/// Every field is serde-defaulted so stored recipes without the block load.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GrazingConfig {
    /// Whether bites lower the modifier and growth reads it.
    pub enabled: bool,
    /// Multiplier a consuming bite applies to the cell's modifier.
    pub factor: f32,
    /// Lowest value repeated bites can drive the modifier to.
    pub floor: f32,
    /// Ticks a fully floored modifier needs to recover from 0.0 to 1.0.
    /// Normalized to `[1, 10_000]`; zero falls back to 1000.
    pub recovery_ticks: u32,
}

impl Default for GrazingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            factor: 0.5,
            floor: 0.05,
            recovery_ticks: DEFAULT_GRAZING_RECOVERY_TICKS,
        }
    }
}

const DEFAULT_GRAZING_RECOVERY_TICKS: u32 = 1000;
/// Upper bound `normalize` clamps `recovery_ticks` to; the grazing property
/// tests cover exactly `1..=MAX_GRAZING_RECOVERY_TICKS`.
pub(crate) const MAX_GRAZING_RECOVERY_TICKS: u32 = 10_000;

/// Fertility layer algorithm variants.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FertilityAlgorithm {
    Uniform {
        value: f32,
    },
    Fbm {
        octaves: u32,
        frequency: f32,
        lacunarity: f32,
        persistence: f32,
        seed: Option<u64>,
    },
    PoissonBlobs {
        blob_count: u32,
        min_radius: f32,
        max_radius: f32,
        falloff: f32,
        seed: Option<u64>,
    },
}

impl Default for FertilityAlgorithm {
    fn default() -> Self {
        FertilityAlgorithm::PoissonBlobs {
            blob_count: 900,
            min_radius: 5.0,
            max_radius: 15.0,
            falloff: 0.5,
            seed: None,
        }
    }
}

/// A single weighted fertility layer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FertilityLayer {
    pub algorithm: FertilityAlgorithm,
    /// Blend weight for this layer. Default: 1.0.
    pub weight: f32,
    #[serde(default)]
    pub target: FertilityLayerTarget,
}

impl Default for FertilityLayer {
    fn default() -> Self {
        Self {
            algorithm: FertilityAlgorithm::default(),
            weight: 1.0,
            target: FertilityLayerTarget::default(),
        }
    }
}

/// Target selector for a fertility layer.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FertilityLayerTarget {
    #[default]
    AllFoods,
    SingleType {
        type_idx: OrdinaryFoodTypeId,
    },
}

/// Fertility map configuration governing per-cell food growth multipliers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FertilityConfig {
    /// Whether the fertility map is applied during food growth. Default: true.
    pub enabled: bool,
    /// Minimum fertility value after normalization. Default: 0.0.
    pub min_fertility: f32,
    /// Maximum fertility value after normalization. Default: 2.0.
    pub max_fertility: f32,
    /// Ordered list of weighted algorithm layers. Default: two PoissonBlobs layers.
    pub layers: Vec<FertilityLayer>,
}

impl Default for FertilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_fertility: 0.0,
            max_fertility: 2.0,
            layers: vec![FertilityLayer::default(), FertilityLayer::default()],
        }
    }
}

/// Full food config: shared substrate settings plus configured food types and fertility settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodConfig {
    #[serde(default)]
    pub shared: FoodResourceConfig,
    #[serde(default = "default_food_types")]
    pub types: Vec<FoodTypeConfig>,
    #[serde(default)]
    pub fertility: FertilityConfig,
    #[serde(default)]
    pub annealing: AnnealingConfig,
}

fn default_food_types() -> Vec<FoodTypeConfig> {
    vec![FoodTypeConfig::default()]
}

impl Default for FoodConfig {
    fn default() -> Self {
        Self {
            shared: FoodResourceConfig::default(),
            types: default_food_types(),
            fertility: FertilityConfig::default(),
            annealing: AnnealingConfig::default(),
        }
    }
}

impl FoodConfig {
    /// Wrap a shared food substrate config into a single-type food config.
    ///
    /// The one synthesized food type inherits the shared initial density and
    /// coverage, and the top-level fertility and annealing settings mirror the
    /// shared ones, so a world configured this way has exactly one ordinary-food
    /// type. Used for placeholder configs and for tests that only vary the
    /// shared substrate.
    #[must_use]
    pub fn single_type(shared: FoodResourceConfig) -> Self {
        let primary_type = FoodTypeConfig {
            initial_density: shared.initial_density,
            initial_coverage: shared.initial_coverage,
            ..FoodTypeConfig::default()
        };
        Self {
            types: vec![primary_type],
            fertility: shared.fertility.clone(),
            annealing: shared.annealing.clone(),
            shared,
        }
    }
}

impl std::ops::Deref for FoodConfig {
    type Target = FoodResourceConfig;

    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}

impl std::ops::DerefMut for FoodConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shared
    }
}

/// Annealing configuration that ramps fertility bounds over time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnealingConfig {
    /// Whether annealing is active. Default: false.
    pub enabled: bool,
    /// Number of ticks over which to ramp from initial to final bounds. Default: 5000.
    pub ramp_ticks: u64,
    /// Minimum fertility at tick 0 (before annealing completes). Default: 0.3.
    pub initial_min_fertility: f32,
    /// Maximum fertility at tick 0 (before annealing completes). Default: 1.5.
    pub initial_max_fertility: f32,
}

impl Default for AnnealingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ramp_ticks: 5000,
            initial_min_fertility: 0.3,
            initial_max_fertility: 1.5,
        }
    }
}

/// One additive startup barrier layer, using the existing pattern schema.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainLayer {
    pub params: crate::patterns::PatternParams,
    pub bounds: Option<crate::patterns::PatternBounds>,
    pub seed: Option<u64>,
}

/// World/grid config. Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldConfig {
    pub width: u16,
    pub height: u16,
    pub edge_mode: WorldEdgeMode,
    #[serde(default)]
    pub terrain: Vec<TerrainLayer>,
    #[serde(default)]
    pub world_seed: Option<u64>,
    pub food: FoodConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 1600,
            height: 1600,
            edge_mode: WorldEdgeMode::default(),
            terrain: Vec::new(),
            world_seed: None,
            food: FoodConfig::default(),
        }
    }
}

/// Founder genome profile used when seeding startup creatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FounderProfile {
    /// Canonical v3alpha1 founder genome.
    #[default]
    V3Alpha1,
    /// Founder variant that prioritizes foraging before reproduction.
    ForageFirstSparse,
    /// Forage-first founder with a higher reproduction gate to delay early branching.
    ForageFirstSparseConservative,
    /// Forage-first founder that transfers more energy to offspring.
    ForageFirstSparseRichOffspring,
    /// Forage-first founder with a mid-threshold, mid-transfer reproduction policy.
    ForageFirstSparseBalanced,
}

/// Energy lifecycle config. Canonical owner: v3-runtime-config-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyLifecycleConfig {
    pub initial_energy: f32,
    pub max_energy: f32,
    pub energy_decay_per_tick: f32,
    pub min_reproduce_energy: f32,
    #[serde(default = "default_min_reproduce_age")]
    pub min_reproduce_age: u64,
    /// Span over which a creature's `AgeTicks` input rises from 0 to 1:
    /// the brain reads `min(age / age_reference_ticks, 1)`. `0` normalizes to
    /// the default.
    #[serde(default = "default_age_reference_ticks")]
    pub age_reference_ticks: u64,
    pub default_offspring_energy: f32,
    /// Per-tick maintenance charge on every unit of `genome_size()`, settled in
    /// Phase 0 beside `energy_decay_per_tick`. `0.0` disables the charge.
    #[serde(default = "default_genome_carry_cost_per_unit")]
    pub genome_carry_cost_per_unit: f32,
    /// Per-birth surcharge on the parent's reproduce charge: the Step 5 charge
    /// is multiplied by `1 + rate * max(genome_size() - founder size, 0)`, so
    /// the founder pays the base charge and every unit above it costs more to
    /// copy. `0.0` disables the surcharge.
    #[serde(default = "default_genome_replication_cost_per_unit")]
    pub genome_replication_cost_per_unit: f32,
}

fn default_min_reproduce_age() -> u64 {
    20
}

fn default_age_reference_ticks() -> u64 {
    500
}

fn default_genome_carry_cost_per_unit() -> f32 {
    1e-4
}

fn default_genome_replication_cost_per_unit() -> f32 {
    0.1
}

impl Default for EnergyLifecycleConfig {
    fn default() -> Self {
        Self {
            initial_energy: 20.0,
            max_energy: 200.0,
            energy_decay_per_tick: 0.5,
            min_reproduce_energy: 30.0,
            min_reproduce_age: default_min_reproduce_age(),
            age_reference_ticks: default_age_reference_ticks(),
            default_offspring_energy: 100.0,
            genome_carry_cost_per_unit: default_genome_carry_cost_per_unit(),
            genome_replication_cost_per_unit: default_genome_replication_cost_per_unit(),
        }
    }
}

/// Energy action costs config. Canonical owner: v3-runtime-config-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyCostsConfig {
    pub move_cost: f32,
    pub eat_cost: f32,
    pub eat_reward_per_food: f32,
    pub noop_cost: f32,
    pub reproduce_cost: f32,
    /// Additional energy penalty applied when an action fails (move blocked, eat empty cell, etc.).
    pub failed_action_penalty: f32,
}

impl Default for EnergyCostsConfig {
    fn default() -> Self {
        Self {
            move_cost: 0.2,
            eat_cost: 0.0,
            eat_reward_per_food: 5.0,
            noop_cost: 0.05,
            reproduce_cost: 0.1,
            failed_action_penalty: 1.0,
        }
    }
}

/// Complexity-based energy cost multiplier config.
/// Operates on functional complexity (reachability-aware), not total genome size.
/// Canonical owner: v3-runtime-config-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexityEnergyCostConfig {
    /// Whether the complexity energy cost multiplier is active.
    pub enabled: bool,
    /// Genome complexity score at or below which no penalty applies.
    pub threshold: u32,
    /// Multiplier per complexity point above the threshold.
    pub scaling_factor: f32,
}

impl Default for ComplexityEnergyCostConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 50,
            scaling_factor: 0.002,
        }
    }
}

impl ComplexityEnergyCostConfig {
    /// Returns the energy cost multiplier for a creature with the given complexity score.
    ///
    /// Formula: `1.0 + max(0, complexity - threshold) * scaling_factor`
    /// Returns 1.0 (no penalty) when disabled or complexity is at or below threshold.
    #[inline]
    #[must_use]
    pub fn multiplier(&self, complexity: u32) -> f32 {
        if !self.enabled || complexity <= self.threshold {
            return 1.0;
        }
        1.0 + (complexity - self.threshold) as f32 * self.scaling_factor
    }
}

/// Age-based energy cost multiplier config.
/// Canonical owner: v3-runtime-config-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeEnergyCostConfig {
    /// Whether the age energy cost multiplier is active.
    pub enabled: bool,
    /// Age (in ticks) through which no age-based multiplier applies.
    #[serde(default = "default_age_cost_grace_ticks")]
    pub grace_ticks: u64,
    /// Age (in ticks) at which the maximum multiplier applies.
    pub age_cap: u64,
    /// Maximum energy cost multiplier at or beyond age_cap.
    pub max_multiplier: f32,
}

impl Default for AgeEnergyCostConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            grace_ticks: default_age_cost_grace_ticks(),
            age_cap: 200,
            max_multiplier: 10.0,
        }
    }
}

fn default_age_cost_grace_ticks() -> u64 {
    100
}

impl AgeEnergyCostConfig {
    /// Returns the energy cost multiplier for a creature of the given age.
    ///
    /// Formula after the grace period:
    /// `1.0 + (max_multiplier - 1.0) * min(1.0, (age - grace_ticks) / (age_cap - grace_ticks))^2`.
    ///
    /// Returns 1.0 when disabled, `age_cap` is 0, or age is within the grace period. If
    /// `age_cap` is not later than the grace period, the maximum applies immediately after grace.
    #[inline]
    #[must_use]
    pub fn multiplier(&self, age: u64) -> f32 {
        if !self.enabled || self.age_cap == 0 || age <= self.grace_ticks {
            return 1.0;
        }
        let ramp_ticks = self.age_cap.saturating_sub(self.grace_ticks);
        if ramp_ticks == 0 {
            return self.max_multiplier;
        }
        let elapsed_ticks = age.saturating_sub(self.grace_ticks);
        let ratio = (elapsed_ticks as f64 / ramp_ticks as f64).min(1.0) as f32;
        1.0 + (self.max_multiplier - 1.0) * ratio * ratio
    }
}

/// Combined energy config.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyConfig {
    pub lifecycle: EnergyLifecycleConfig,
    pub costs: EnergyCostsConfig,
    #[serde(default)]
    pub complexity_cost: ComplexityEnergyCostConfig,
    #[serde(default)]
    pub age_cost: AgeEnergyCostConfig,
}

impl EnergyConfig {
    /// Combined multiplier for all action energy costs.
    ///
    /// Composes complexity-based and age-based multipliers multiplicatively.
    #[inline]
    #[must_use]
    pub fn action_cost_multiplier(&self, complexity: u32, age: u64) -> f32 {
        self.complexity_cost.multiplier(complexity) * self.age_cost.multiplier(age)
    }

    /// Returns the complexity-and-age-adjusted cost for a given base cost.
    ///
    /// Formula: `base_cost * action_cost_multiplier(complexity, age)`
    #[inline]
    #[must_use]
    pub fn adjusted_action_cost(&self, base_cost: f32, complexity: u32, age: u64) -> f32 {
        base_cost * self.action_cost_multiplier(complexity, age)
    }
}

/// Startup ramp for failed action penalty.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailedActionPenaltyRampConfig {
    pub enabled: bool,
    pub start: f32,
    pub end: f32,
    pub target_tick: u64,
}

impl Default for FailedActionPenaltyRampConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            start: 0.0,
            end: 1.0,
            target_tick: 62680,
        }
    }
}

impl FailedActionPenaltyRampConfig {
    /// Returns true while interpolation should still be applied.
    #[must_use]
    pub fn is_active(&self, tick: u64) -> bool {
        let target_tick = self.target_tick.max(1);
        self.enabled && tick < target_tick
    }

    /// Returns linearly interpolated ramp value for a tick.
    #[must_use]
    pub fn value_for_tick(&self, tick: u64) -> f32 {
        let target_tick = self.target_tick.max(1);
        let progress = (tick as f32 / target_tick as f32).clamp(0.0, 1.0);
        self.start + (self.end - self.start) * progress
    }
}

/// Startup-time ramp controls for values that should vary early in a run.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupRampsConfig {
    #[serde(default)]
    pub failed_action_penalty: FailedActionPenaltyRampConfig,
}

/// Startup-only controls persisted in config and applied on restart/startup.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupConfig {
    #[serde(default)]
    pub ramps: StartupRampsConfig,
}

/// VM runtime config. Canonical owner: v3-runtime-config-spec.md Section 2.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VmRuntimeConfig {
    pub opcode_cost_multiplier: f32,
    /// Free instructions per VM node dispatch before the activity ramp applies.
    /// Default 100. Any value is valid; 0 ramps from the first instruction.
    #[serde(default = "default_step_ramp_allowance")]
    pub step_ramp_allowance: u32,
    /// Extra energy charged per excess step, per step past the allowance, within
    /// one VM node dispatch. Default 1e-6; 0.0 disables the ramp.
    #[serde(default = "default_step_ramp_cost")]
    pub step_ramp_cost: f32,
}

fn default_step_ramp_allowance() -> u32 {
    100
}

fn default_step_ramp_cost() -> f32 {
    1e-6
}

impl Default for VmRuntimeConfig {
    fn default() -> Self {
        Self {
            opcode_cost_multiplier: 1e-6,
            step_ramp_allowance: default_step_ramp_allowance(),
            step_ramp_cost: default_step_ramp_cost(),
        }
    }
}

/// Perception runtime config. Canonical owner: v3-runtime-config-spec.md.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerceptionRuntimeConfig {
    /// Vision radius for extended perception area summaries. Default 5, valid 1..=8.
    #[serde(default = "default_vision_radius")]
    pub vision_radius: u8,
}

fn default_vision_radius() -> u8 {
    5
}

impl Default for PerceptionRuntimeConfig {
    fn default() -> Self {
        Self {
            vision_radius: default_vision_radius(),
        }
    }
}

/// Mesh/VM/graph execution limits. Canonical owner: v3-runtime-config-spec.md Section 2.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    pub max_mesh_hops: u32,
    pub max_vm_steps: u32,
    /// Retained and validated, but inactive since the T11.F06 world-tick clock.
    pub max_graph_relax_iters: u32,
    /// Retained inactive convergence setting.
    pub graph_convergence_epsilon: f32,
    /// Retained inactive convergence setting.
    pub graph_convergence_stable_passes: u32,
    pub graph_node_base_cost: f32,
    /// Energy cost per plasticity weight update. Default 0.0 (free during initial rollout).
    #[serde(default, alias = "hebbian_update_cost")]
    pub plasticity_update_cost: f32,
    /// Energy cost per reward-modulated weight update. Default 0.0 (free during initial rollout).
    #[serde(default)]
    pub reward_learning_cost: f32,
    /// Maximum number of actions a creature can queue per turn.
    #[serde(default = "default_max_actions_per_turn")]
    pub max_actions_per_turn: usize,
    pub vm: VmRuntimeConfig,
    /// Extended perception config.
    #[serde(default)]
    pub perception: PerceptionRuntimeConfig,
}

fn default_max_actions_per_turn() -> usize {
    10
}

fn default_action_queue_cap() -> usize {
    4
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_mesh_hops: 1024,
            max_vm_steps: 10000,
            max_graph_relax_iters: 15,
            graph_convergence_epsilon: 1e-3,
            graph_convergence_stable_passes: 2,
            graph_node_base_cost: 1e-5,
            plasticity_update_cost: 0.0,
            reward_learning_cost: 0.0,
            max_actions_per_turn: default_max_actions_per_turn(),
            vm: VmRuntimeConfig::default(),
            perception: PerceptionRuntimeConfig::default(),
        }
    }
}

/// Phenotype mutation tuning config. Canonical owner: v3-phenotype-spec.md Section 6.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhenotypeConfig {
    /// RGB channel step magnitude (wrapping u8). Default: 1. Falls back to 1 if 0.
    pub channel_step: u8,
    /// Probability of switching active channel per reproduction. Default: 0.001. Clamped [0.0, 1.0].
    pub channel_change_chance: f32,
    /// Probability of flipping the selected channel's polarity. Default: 0.0002. Clamped [0.0, 1.0].
    pub polarity_flip_chance: f32,
}

impl Default for PhenotypeConfig {
    fn default() -> Self {
        Self {
            channel_step: 1,
            channel_change_chance: 0.001,
            polarity_flip_chance: 0.0002,
        }
    }
}

/// Per-domain bias toward reachable (functional) mesh nodes during mutation target selection.
///
/// Each field is a probability in [0.0, 1.0]. At 0.0, selection is uniform over all eligible
/// nodes (current behavior). At 1.0, mutations always target reachable nodes when possible.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReachableBiasConfig {
    pub topology: f64,
    pub vm: f64,
    pub graph: f64,
    pub input_ref: f64,
}

impl Default for ReachableBiasConfig {
    fn default() -> Self {
        Self {
            topology: 0.0,
            vm: 0.0,
            graph: 0.0,
            input_ref: 0.0,
        }
    }
}

/// Mutation tuning config. Canonical owner: v3-runtime-config-spec.md Section 3.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationConfig {
    /// Production supply rule (T11.F19): one Bernoulli trial at `per_unit_rate`
    /// per `genome_size()` unit of the parent, so the requested event count is
    /// `Binomial(genome_size(), per_unit_rate)`. When `false`, the four
    /// per-birth fields below run as the disabled legacy rule.
    #[serde(default = "default_per_unit_supply_enabled")]
    pub per_unit_supply_enabled: bool,
    /// Chance that one genome unit requests a mutation event at birth.
    #[serde(default = "default_per_unit_rate")]
    pub per_unit_rate: f64,
    /// Legacy per-birth rule: trigger roll. Runs only when
    /// `per_unit_supply_enabled` is `false`.
    pub mutation_probability: f64,
    pub per_birth_mutation_events_min: u32,
    pub per_birth_mutation_events_max: u32,
    /// Chance to request another event after the minimum, up to the maximum.
    #[serde(default = "default_mutation_event_continuation_probability")]
    pub per_birth_mutation_event_continuation_probability: f64,
    /// Probability of selecting the mesh (Topology) layer per mutation event.
    /// Complement (1 - this) selects the node-internal layer (VM/Graph/InputRef).
    pub mesh_layer_probability: f64,
    /// Relative weight of node and mesh-slice copying, as a percentage of
    /// their base weights. 25 quarters copying; 100 restores base weights;
    /// 0 disables these operators. Other operators retain their relative weights.
    #[serde(default = "default_large_copy_weight_percent")]
    pub large_copy_weight_percent: u8,
    /// Total genome size above which pressure escalates against structural growth.
    #[serde(alias = "complexity_cap")]
    pub genome_size_cap: u32,
    /// Whether the genome size pressure gate is active.
    #[serde(alias = "complexity_pressure_enabled")]
    pub genome_size_pressure_enabled: bool,
    /// Capacity of the action queue. Compound input fan-out counts depend on this.
    #[serde(default = "default_action_queue_cap")]
    pub action_queue_cap: usize,
    pub phenotype: PhenotypeConfig,
    #[serde(default)]
    pub reachable_bias: ReachableBiasConfig,
    /// Probability that a mutation target is drawn from the nodes the parent's
    /// brain dispatched within `executed_window_ticks` of its last tick, in
    /// every domain. The residual keeps drawing from the whole eligible set.
    #[serde(default = "default_executed_bias")]
    pub executed_bias: f64,
    /// How many ticks back a dispatch still counts as "recently executed".
    #[serde(default = "default_executed_window_ticks")]
    pub executed_window_ticks: u64,
}

impl MutationConfig {
    /// This config on the legacy per-birth supply rule: `per_unit_supply_enabled`
    /// forced `false`, every other field kept. The fixed-count control the
    /// drift walk and `recruitment_paths` run on (T11.F19).
    #[must_use]
    pub fn with_legacy_supply(mut self) -> Self {
        self.per_unit_supply_enabled = false;
        self
    }
}

fn default_per_unit_supply_enabled() -> bool {
    true
}

/// Sized so the 111-unit V3Alpha1 founder keeps about 0.55 requested events
/// per birth; the founder pin test holds the two together.
fn default_per_unit_rate() -> f64 {
    0.005
}

fn default_mutation_event_continuation_probability() -> f64 {
    0.2
}

fn default_executed_bias() -> f64 {
    0.9
}

fn default_large_copy_weight_percent() -> u8 {
    25
}

fn default_executed_window_ticks() -> u64 {
    100
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            per_unit_supply_enabled: default_per_unit_supply_enabled(),
            per_unit_rate: default_per_unit_rate(),
            mutation_probability: 0.44,
            per_birth_mutation_events_min: 1,
            per_birth_mutation_events_max: 10,
            per_birth_mutation_event_continuation_probability:
                default_mutation_event_continuation_probability(),
            mesh_layer_probability: 0.2,
            large_copy_weight_percent: default_large_copy_weight_percent(),
            genome_size_cap: 1200,
            genome_size_pressure_enabled: false,
            action_queue_cap: 4,
            phenotype: PhenotypeConfig::default(),
            reachable_bias: ReachableBiasConfig::default(),
            executed_bias: default_executed_bias(),
            executed_window_ticks: default_executed_window_ticks(),
        }
    }
}

/// Action log configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionLogConfig {
    /// Maximum entries per creature. Default: 500.
    pub capacity: usize,
}

impl Default for ActionLogConfig {
    fn default() -> Self {
        Self { capacity: 500 }
    }
}

/// Predation config for the StealEnergy action.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredationConfig {
    /// Fraction of attempted steal amount paid as attacker cost.
    pub steal_cost_rate: f32,
    /// Energy bonus per unit of victim genome complexity on kill.
    pub kill_complexity_bonus_multiplier: f32,
}

impl Default for PredationConfig {
    fn default() -> Self {
        Self {
            steal_cost_rate: 0.05,
            kill_complexity_bonus_multiplier: 0.0,
        }
    }
}

/// Shared memory config for creature-level persistent f32 slot memory.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SharedMemoryConfig {
    /// Global decay rate applied to all shared memory slots at tick start.
    /// Clamped to [0.0, 1.0]. Default 0.0 (no decay).
    pub decay_rate: f32,
}

impl Default for SharedMemoryConfig {
    fn default() -> Self {
        Self { decay_rate: 0.0 }
    }
}

/// Population caps. Canonical owner: v3-runtime-config-spec.md Section 5.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationConfig {
    pub initial_creatures: u32,
    pub max_creatures: u32,
    #[serde(default)]
    pub founder_profile: FounderProfile,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            initial_creatures: 10000,
            max_creatures: 100000,
            founder_profile: FounderProfile::default(),
        }
    }
}

/// Full simulation configuration.
/// Use `SimulationConfig::default()` to get canonical spec defaults.
/// Call `normalize()` after deserialization or programmatic construction to
/// apply fallback rules for out-of-range values.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationConfig {
    pub world: WorldConfig,
    pub energy: EnergyConfig,
    #[serde(default)]
    pub startup: StartupConfig,
    pub runtime: RuntimeConfig,
    pub mutation: MutationConfig,
    pub population: PopulationConfig,
    #[serde(default)]
    pub predation: PredationConfig,
    #[serde(default)]
    pub action_log: ActionLogConfig,
    #[serde(default)]
    pub shared_memory: SharedMemoryConfig,
}

impl SimulationConfig {
    /// Apply startup-time config overrides that should pin runtime-visible values.
    pub fn apply_startup_overrides(&mut self) {
        let ramp = &self.startup.ramps.failed_action_penalty;
        if ramp.enabled {
            self.energy.costs.failed_action_penalty = ramp.end;
        }
    }

    /// Whether the startup failed action penalty ramp is currently active.
    #[must_use]
    pub fn failed_action_penalty_ramp_active(&self, tick: u64) -> bool {
        self.startup.ramps.failed_action_penalty.is_active(tick)
    }

    /// Effective failed action penalty for a given tick.
    ///
    /// During an active startup ramp, this interpolates from `start` to `end`.
    /// Otherwise it returns the runtime config value.
    #[must_use]
    pub fn failed_action_penalty_for_tick(&self, tick: u64) -> f32 {
        let ramp = &self.startup.ramps.failed_action_penalty;
        if ramp.is_active(tick) {
            ramp.value_for_tick(tick)
        } else {
            self.energy.costs.failed_action_penalty
        }
    }

    /// Apply normalization/fallback for out-of-range values per spec constraints.
    pub fn normalize(&mut self) {
        for layer in &mut self.world.terrain {
            layer.params.normalize();
        }
        normalize_world_food(&mut self.world);

        let el = &mut self.energy.lifecycle;
        el.initial_energy = normalize_f32_finite_nonneg(el.initial_energy, 20.0);
        el.max_energy = normalize_f32_finite_min(el.max_energy, 1.0, 200.0);
        el.energy_decay_per_tick = normalize_f32_finite_nonneg(el.energy_decay_per_tick, 0.5);
        el.min_reproduce_energy = normalize_f32_finite_nonneg(el.min_reproduce_energy, 30.0);
        if el.age_reference_ticks == 0 {
            el.age_reference_ticks = default_age_reference_ticks();
        }
        el.default_offspring_energy =
            normalize_f32_finite_nonneg(el.default_offspring_energy, 100.0);
        el.genome_carry_cost_per_unit = normalize_f32_finite_nonneg(
            el.genome_carry_cost_per_unit,
            default_genome_carry_cost_per_unit(),
        );
        el.genome_replication_cost_per_unit = normalize_f32_finite_nonneg(
            el.genome_replication_cost_per_unit,
            default_genome_replication_cost_per_unit(),
        );

        let ec = &mut self.energy.costs;
        ec.move_cost = normalize_f32_finite_nonneg(ec.move_cost, 0.2);
        ec.eat_reward_per_food = normalize_f32_finite_nonneg(ec.eat_reward_per_food, 5.0);
        ec.eat_cost = normalize_f32_finite_nonneg(ec.eat_cost, 0.0);
        ec.noop_cost = normalize_f32_finite_nonneg(ec.noop_cost, 0.05);
        ec.reproduce_cost = normalize_f32_finite_nonneg(ec.reproduce_cost, 0.1);
        ec.failed_action_penalty = normalize_f32_finite_nonneg(ec.failed_action_penalty, 1.0);
        let failed_penalty_fallback = ec.failed_action_penalty;

        let ramp = &mut self.startup.ramps.failed_action_penalty;
        ramp.start = normalize_f32_finite_nonneg(ramp.start, failed_penalty_fallback);
        ramp.end = normalize_f32_finite_nonneg(ramp.end, failed_penalty_fallback);
        if ramp.target_tick < 1 {
            ramp.target_tick = 1;
        }

        let cc = &mut self.energy.complexity_cost;
        cc.scaling_factor = normalize_f32_finite_nonneg(cc.scaling_factor, 0.002);

        let ac = &mut self.energy.age_cost;
        ac.max_multiplier = normalize_f32_finite_min(ac.max_multiplier, 1.0, 10.0);

        let rt = &mut self.runtime;
        if rt.max_mesh_hops < 1 {
            rt.max_mesh_hops = 1024;
        }
        if rt.max_vm_steps < 1 {
            rt.max_vm_steps = 10000;
        }
        if rt.max_graph_relax_iters < 1 {
            rt.max_graph_relax_iters = 15;
        }
        rt.graph_convergence_epsilon = normalize_f32_nonneg(rt.graph_convergence_epsilon, 1e-3);
        if rt.graph_convergence_stable_passes < 1 {
            rt.graph_convergence_stable_passes = 2;
        }
        rt.graph_node_base_cost = normalize_f32_nonneg(rt.graph_node_base_cost, 1e-5);
        rt.plasticity_update_cost = normalize_f32_finite_nonneg(rt.plasticity_update_cost, 0.0);
        rt.reward_learning_cost = normalize_f32_finite_nonneg(rt.reward_learning_cost, 0.0);
        if rt.max_actions_per_turn < 1 {
            rt.max_actions_per_turn = 10;
        }
        rt.vm.opcode_cost_multiplier =
            normalize_f32_finite_nonneg(rt.vm.opcode_cost_multiplier, 1e-6);
        rt.vm.step_ramp_cost =
            normalize_f32_finite_nonneg(rt.vm.step_ramp_cost, default_step_ramp_cost());
        rt.perception.vision_radius = rt.perception.vision_radius.clamp(1, 8);

        let m = &mut self.mutation;
        m.per_unit_rate = if m.per_unit_rate.is_finite() {
            m.per_unit_rate.clamp(0.0, 1.0)
        } else {
            default_per_unit_rate()
        };
        // per_unit_supply_enabled: bool, no normalization needed.
        m.mutation_probability = m.mutation_probability.clamp(0.0, 1.0);
        m.mesh_layer_probability = m.mesh_layer_probability.clamp(0.0, 1.0);
        m.large_copy_weight_percent = m.large_copy_weight_percent.min(100);
        let continuation = m.per_birth_mutation_event_continuation_probability;
        m.per_birth_mutation_event_continuation_probability = if continuation.is_finite() {
            continuation.clamp(0.0, 1.0)
        } else {
            default_mutation_event_continuation_probability()
        };
        if m.per_birth_mutation_events_min < 1 {
            m.per_birth_mutation_events_min = 1;
        }
        if m.per_birth_mutation_events_max < m.per_birth_mutation_events_min {
            m.per_birth_mutation_events_max = m.per_birth_mutation_events_min;
        }
        // Cap must be >= 1, <= runtime queue capacity, and <= u16/3 to keep
        // ActionQueue compound width (`cap * 3`) representable in u16.
        let max_action_queue_cap = rt.max_actions_per_turn.min(21845);
        m.action_queue_cap = m.action_queue_cap.clamp(1, max_action_queue_cap);
        // genome_size_cap: 0 disables pressure (handled by is_restricted), no normalization needed.
        // genome_size_pressure_enabled: bool, no normalization needed.
        let ph = &mut m.phenotype;
        if ph.channel_step == 0 {
            ph.channel_step = 1;
        }
        ph.channel_change_chance = ph.channel_change_chance.clamp(0.0, 1.0);
        ph.polarity_flip_chance = ph.polarity_flip_chance.clamp(0.0, 1.0);

        let rb = &mut m.reachable_bias;
        rb.topology = if rb.topology.is_finite() {
            rb.topology.clamp(0.0, 1.0)
        } else {
            0.0
        };
        rb.vm = if rb.vm.is_finite() {
            rb.vm.clamp(0.0, 1.0)
        } else {
            0.0
        };
        rb.graph = if rb.graph.is_finite() {
            rb.graph.clamp(0.0, 1.0)
        } else {
            0.0
        };
        rb.input_ref = if rb.input_ref.is_finite() {
            rb.input_ref.clamp(0.0, 1.0)
        } else {
            0.0
        };

        m.executed_bias = if m.executed_bias.is_finite() {
            m.executed_bias.clamp(0.0, 1.0)
        } else {
            default_executed_bias()
        };
        if m.executed_window_ticks == 0 {
            m.executed_window_ticks = default_executed_window_ticks();
        }

        if self.action_log.capacity < 1 {
            self.action_log.capacity = 500;
        }

        let pred = &mut self.predation;
        pred.steal_cost_rate = normalize_f32_clamp(pred.steal_cost_rate, 0.0, 1.0, 0.05);
        pred.kill_complexity_bonus_multiplier =
            normalize_f32_finite_nonneg(pred.kill_complexity_bonus_multiplier, 0.0);

        self.shared_memory.decay_rate =
            normalize_f32_clamp(self.shared_memory.decay_rate, 0.0, 1.0, 0.0);

        let p = &mut self.population;
        if p.initial_creatures < 1 {
            p.initial_creatures = 10000;
        }
        if p.max_creatures < p.initial_creatures {
            p.max_creatures = 100000;
        }
    }
}

fn normalize_world_food(world: &mut WorldConfig) {
    if world.width == 0 {
        world.width = 1600;
    }
    if world.height == 0 {
        world.height = 1600;
    }
    normalize_food_config(&mut world.food);
}

fn normalize_food_config(food: &mut FoodConfig) {
    normalize_food_shared(&mut food.shared);
    normalize_food_types(food);
    sync_shared_from_canonical_food(food);
    normalize_food_layer_targets(food);
}

fn normalize_food_shared(shared: &mut FoodResourceConfig) {
    shared.max_density = normalize_f32_finite_positive(shared.max_density, 1.0);
    shared.growth_rate = normalize_f32_clamp(shared.growth_rate, 0.0, 1.0, 0.09);
    shared.initial_coverage = normalize_f32_clamp(shared.initial_coverage, 0.0, 1.0, 0.54);
    shared.spread_threshold_ratio =
        normalize_f32_clamp(shared.spread_threshold_ratio, 0.0, 1.0, 0.8);
    shared.spread_density_ratio = normalize_f32_clamp(shared.spread_density_ratio, 0.0, 1.0, 0.25);
    shared.recovery_spawn_rate = normalize_f32_clamp(shared.recovery_spawn_rate, 0.0, 1.0, 0.01);
    shared.recovery_floor_ratio = normalize_f32_clamp(shared.recovery_floor_ratio, 0.0, 1.0, 0.01);
    shared.occupancy_depletion.deposit_per_occupied_tick = normalize_f32_clamp(
        shared.occupancy_depletion.deposit_per_occupied_tick,
        0.0,
        1.0,
        0.08,
    );
    shared.grazing.factor = normalize_f32_clamp(shared.grazing.factor, 0.0, 1.0, 0.5);
    shared.grazing.floor = normalize_f32_clamp(shared.grazing.floor, 0.0, 1.0, 0.05);
    shared.grazing.recovery_ticks = match shared.grazing.recovery_ticks {
        0 => DEFAULT_GRAZING_RECOVERY_TICKS,
        ticks => ticks.min(MAX_GRAZING_RECOVERY_TICKS),
    };
    shared.initial_density = normalize_f32_clamp(
        shared.initial_density,
        0.0,
        shared.max_density,
        shared.max_density,
    );
}

fn normalize_food_types(food: &mut FoodConfig) {
    if food.types.is_empty() {
        food.types = default_food_types();
    }

    let max_density = food.shared.max_density;
    for food_type in &mut food.types {
        food_type.initial_coverage =
            normalize_f32_clamp(food_type.initial_coverage, 0.0, 1.0, 0.54);
        food_type.initial_density =
            normalize_f32_clamp(food_type.initial_density, 0.0, max_density, max_density);
        food_type.growth_inhibitor = normalize_f32_clamp(food_type.growth_inhibitor, 0.0, 1.0, 0.2);
        food_type.energy_per_unit = food_type
            .energy_per_unit
            .filter(|v| v.is_finite())
            .map(|v| v.max(0.0));
        food_type.growth_rate = food_type
            .growth_rate
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.0, 1.0));
        food_type.recovery_spawn_rate = food_type
            .recovery_spawn_rate
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.0, 1.0));
    }
}

fn sync_shared_from_canonical_food(food: &mut FoodConfig) {
    if let Some(primary_type) = food.types.first() {
        food.shared.initial_density = primary_type.initial_density;
        food.shared.initial_coverage = primary_type.initial_coverage;
    }
    food.shared.fertility = food.fertility.clone();
    food.shared.annealing = food.annealing.clone();
}

fn normalize_food_layer_targets(food: &mut FoodConfig) {
    for layer in &mut food.fertility.layers {
        if let FertilityLayerTarget::SingleType { type_idx } = layer.target {
            if usize::from(type_idx.get()) >= food.types.len() {
                layer.target = FertilityLayerTarget::AllFoods;
            }
        }
    }
}

fn normalize_f32_clamp(v: f32, lo: f32, hi: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v.clamp(lo, hi)
    } else {
        fallback
    }
}

fn normalize_f32_finite_nonneg(v: f32, fallback: f32) -> f32 {
    if v.is_finite() && v >= 0.0 {
        v
    } else {
        fallback
    }
}

fn normalize_f32_finite_min(v: f32, min: f32, fallback: f32) -> f32 {
    if v.is_finite() && v >= min {
        v
    } else {
        fallback
    }
}

fn normalize_f32_nonneg(v: f32, fallback: f32) -> f32 {
    if v >= 0.0 {
        v
    } else {
        fallback
    }
}

fn normalize_f32_finite_positive(v: f32, fallback: f32) -> f32 {
    if v.is_finite() && v > 0.0 {
        v
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn energy_only_default_contract() {
        let config = SimulationConfig::default();
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(config.world.food.types.len(), 1);
        assert_eq!(config.world.food.types[0].name, "Primary Food");
        assert_eq!(config.world.food.types[0].color, "#22c55e");
        assert_eq!(json["energy"]["costs"]["eat_reward_per_food"], 5.0);
        assert!(json.get("nutrition").is_none());
        assert!(json["world"]["food"]["types"][0]
            .get("metabolic_energy_yield")
            .is_none());
    }

    #[test]
    fn shared_reward_normalizes_invalid_values_and_rejects_obsolete_fields() {
        for input in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0] {
            let mut config = SimulationConfig::default();
            config.energy.costs.eat_reward_per_food = input;
            config.normalize();
            assert_eq!(config.energy.costs.eat_reward_per_food, 5.0);
        }
        let mut json = serde_json::to_value(SimulationConfig::default()).unwrap();
        json["nutrition"] = serde_json::json!({});
        assert!(serde_json::from_value::<SimulationConfig>(json).is_err());
        for key in ["metabolic_energy_yield", "reproductive_reserve_yield"] {
            let mut food = serde_json::to_value(FoodTypeConfig::default()).unwrap();
            food[key] = serde_json::json!(1.0);
            assert!(serde_json::from_value::<FoodTypeConfig>(food).is_err());
        }
    }

    proptest! {
        #[test]
        fn shared_reward_normalization_preserves_finite_nonnegative_values(reward in 0.0f32..f32::MAX) {
            let mut config = SimulationConfig::default();
            config.energy.costs.eat_reward_per_food = reward;
            config.normalize();
            prop_assert_eq!(config.energy.costs.eat_reward_per_food, reward);
        }
    }

    proptest! {
        #[test]
        fn action_log_capacity_preserves_positive_values(capacity in 1usize..usize::MAX) {
            for value in [0, 1, capacity] {
                let mut config = SimulationConfig::default();
                config.action_log.capacity = value;
                config.normalize();
                prop_assert_eq!(config.action_log.capacity, if value == 0 { 500 } else { value });
            }
        }
    }

    #[test]
    fn continuation_probability_defaults_when_missing() {
        let mut json = serde_json::to_value(MutationConfig::default()).unwrap();
        json.as_object_mut()
            .unwrap()
            .remove("per_birth_mutation_event_continuation_probability");
        let config: MutationConfig = serde_json::from_value(json).unwrap();
        assert_eq!(
            config.per_birth_mutation_event_continuation_probability,
            0.2
        );
    }

    #[test]
    fn continuation_probability_nonfinite_uses_default() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut config = SimulationConfig::default();
            config
                .mutation
                .per_birth_mutation_event_continuation_probability = value;
            config.normalize();
            assert_eq!(
                config
                    .mutation
                    .per_birth_mutation_event_continuation_probability,
                0.2
            );
        }
    }

    proptest! {
        #[test]
        fn mutation_supply_normalizes_and_roundtrips(
            min in any::<u32>(), max in any::<u32>(), continuation in -2.0f64..3.0,
        ) {
            let mut config = SimulationConfig::default();
            config.mutation.per_birth_mutation_events_min = min;
            config.mutation.per_birth_mutation_events_max = max;
            config.mutation.per_birth_mutation_event_continuation_probability = continuation;
            config.normalize();
            let m = &config.mutation;
            prop_assert_eq!(m.per_birth_mutation_events_min, min.max(1));
            prop_assert_eq!(m.per_birth_mutation_events_max, max.max(min.max(1)));
            prop_assert_eq!(m.per_birth_mutation_event_continuation_probability, continuation.clamp(0.0, 1.0));
            let decoded: MutationConfig = serde_json::from_str(&serde_json::to_string(m).unwrap()).unwrap();
            prop_assert_eq!(decoded.per_birth_mutation_events_min, m.per_birth_mutation_events_min);
            prop_assert_eq!(decoded.per_birth_mutation_events_max, m.per_birth_mutation_events_max);
            prop_assert!((decoded.per_birth_mutation_event_continuation_probability - m.per_birth_mutation_event_continuation_probability).abs() < 1e-15);
        }
    }

    #[test]
    fn per_unit_supply_fields_default_when_missing() {
        let mut json = serde_json::to_value(MutationConfig::default()).unwrap();
        let object = json.as_object_mut().unwrap();
        object.remove("per_unit_supply_enabled");
        object.remove("per_unit_rate");
        let config: MutationConfig = serde_json::from_value(json).unwrap();
        assert!(config.per_unit_supply_enabled);
        assert_eq!(config.per_unit_rate, default_per_unit_rate());
    }

    #[test]
    fn with_legacy_supply_flips_only_the_supply_flag() {
        let config = MutationConfig {
            per_unit_rate: 0.25,
            mutation_probability: 0.75,
            ..MutationConfig::default()
        };
        let mut expected = serde_json::to_value(&config).unwrap();
        expected["per_unit_supply_enabled"] = serde_json::Value::Bool(false);

        let legacy = config.with_legacy_supply();

        assert_eq!(serde_json::to_value(&legacy).unwrap(), expected);
    }

    #[test]
    fn per_unit_rate_nonfinite_uses_default() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut config = SimulationConfig::default();
            config.mutation.per_unit_rate = value;
            config.normalize();
            assert_eq!(config.mutation.per_unit_rate, default_per_unit_rate());
        }
    }

    /// The default rate is pinned to the founder's genome size, so the founder
    /// keeps about 0.55 requested events per birth (T11.F19); the size itself
    /// is pinned by the creature state test, not repeated here.
    #[test]
    fn default_per_unit_rate_keeps_the_founder_near_the_legacy_supply() {
        let founder = crate::creature::founder::v3alpha1_founder_genome();
        let expected = MutationConfig::default().per_unit_rate * f64::from(founder.genome_size());
        assert!(
            (expected - 0.55).abs() <= 0.55 * 0.01,
            "founder expects {expected} requested events per birth"
        );
    }

    proptest! {
        #[test]
        fn per_unit_rate_normalizes_and_roundtrips(rate in -2.0f64..3.0) {
            let mut config = SimulationConfig::default();
            config.mutation.per_unit_rate = rate;
            config.normalize();
            let m = &config.mutation;
            prop_assert_eq!(m.per_unit_rate, rate.clamp(0.0, 1.0));
            let decoded: MutationConfig = serde_json::from_str(&serde_json::to_string(m).unwrap()).unwrap();
            prop_assert!((decoded.per_unit_rate - m.per_unit_rate).abs() < 1e-15);
            prop_assert_eq!(decoded.per_unit_supply_enabled, m.per_unit_supply_enabled);
        }
    }

    #[test]
    fn default_config_matches_spec() {
        let cfg = SimulationConfig::default();
        // World
        assert_eq!(cfg.world.width, 1600);
        assert_eq!(cfg.world.height, 1600);
        assert!(matches!(cfg.world.edge_mode, WorldEdgeMode::Wrap));
        assert!((cfg.world.food.growth_rate - 0.09).abs() < 1e-6);
        assert!((cfg.world.food.initial_density - 1.0).abs() < 1e-6);
        assert!((cfg.world.food.initial_coverage - 0.54).abs() < 1e-6);
        assert!((cfg.world.food.spread_threshold_ratio - 0.8).abs() < 1e-6);
        assert!((cfg.world.food.spread_density_ratio - 0.25).abs() < 1e-6);
        assert!((cfg.world.food.recovery_spawn_rate - 0.01).abs() < 1e-6);
        assert!((cfg.world.food.recovery_floor_ratio - 0.01).abs() < 1e-6);
        assert!((cfg.world.food.max_density - 1.0).abs() < 1e-6);
        assert!(cfg.world.food.occupancy_depletion.enabled);
        assert!((cfg.world.food.occupancy_depletion.deposit_per_occupied_tick - 0.08).abs() < 1e-6);
        assert!(cfg.world.food.fertility.enabled);
        assert_eq!(cfg.world.food.fertility.min_fertility, 0.0);
        assert_eq!(cfg.world.food.fertility.max_fertility, 2.0);
        assert_eq!(cfg.world.food.fertility.layers.len(), 2);
        for layer in &cfg.world.food.fertility.layers {
            assert!((layer.weight - 1.0).abs() < f32::EPSILON);
            match &layer.algorithm {
                FertilityAlgorithm::PoissonBlobs {
                    blob_count,
                    min_radius,
                    max_radius,
                    falloff,
                    seed,
                } => {
                    assert_eq!(*blob_count, 900);
                    assert!((*min_radius - 5.0).abs() < f32::EPSILON);
                    assert!((*max_radius - 15.0).abs() < f32::EPSILON);
                    assert!((*falloff - 0.5).abs() < f32::EPSILON);
                    assert_eq!(*seed, None);
                }
                other => panic!("expected PoissonBlobs default layer, got {other:?}"),
            }
        }
        // Energy lifecycle
        assert!((cfg.energy.lifecycle.initial_energy - 20.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.max_energy - 200.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.energy_decay_per_tick - 0.5).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.min_reproduce_energy - 30.0).abs() < 1e-6);
        assert_eq!(cfg.energy.lifecycle.min_reproduce_age, 20);
        assert!((cfg.energy.lifecycle.default_offspring_energy - 100.0).abs() < 1e-6);
        assert_eq!(cfg.energy.lifecycle.genome_carry_cost_per_unit, 1e-4);
        assert_eq!(cfg.energy.lifecycle.genome_replication_cost_per_unit, 0.1);
        // Complexity energy cost
        assert!(!cfg.energy.complexity_cost.enabled);
        assert_eq!(cfg.energy.complexity_cost.threshold, 50);
        assert!((cfg.energy.complexity_cost.scaling_factor - 0.002).abs() < 1e-6);
        // Age energy cost
        assert!(cfg.energy.age_cost.enabled);
        assert_eq!(cfg.energy.age_cost.grace_ticks, 100);
        assert_eq!(cfg.energy.age_cost.age_cap, 200);
        assert!((cfg.energy.age_cost.max_multiplier - 10.0).abs() < 1e-6);
        // Energy costs
        assert!((cfg.energy.costs.move_cost - 0.2).abs() < 1e-6);
        assert!((cfg.energy.costs.eat_cost - 0.0).abs() < 1e-6);
        assert!((cfg.energy.costs.noop_cost - 0.05).abs() < 1e-6);
        assert!((cfg.energy.costs.reproduce_cost - 0.1).abs() < 1e-6);
        assert!((cfg.energy.costs.failed_action_penalty - 1.0).abs() < 1e-6);
        // Startup ramps
        assert!(cfg.startup.ramps.failed_action_penalty.enabled);
        assert!((cfg.startup.ramps.failed_action_penalty.start - 0.0).abs() < 1e-6);
        assert!((cfg.startup.ramps.failed_action_penalty.end - 1.0).abs() < 1e-6);
        assert_eq!(cfg.startup.ramps.failed_action_penalty.target_tick, 62680);
        // Runtime
        assert_eq!(cfg.runtime.max_mesh_hops, 1024);
        assert_eq!(cfg.runtime.max_vm_steps, 10000);
        assert_eq!(cfg.runtime.max_graph_relax_iters, 15);
        assert!((cfg.runtime.graph_convergence_epsilon - 1e-3).abs() < 1e-6);
        assert_eq!(cfg.runtime.graph_convergence_stable_passes, 2);
        assert!((cfg.runtime.graph_node_base_cost - 1e-5).abs() < 1e-9);
        assert!((cfg.runtime.vm.opcode_cost_multiplier - 1e-6).abs() < 1e-12);
        assert_eq!(cfg.runtime.vm.step_ramp_allowance, 100);
        assert!((cfg.runtime.vm.step_ramp_cost - 1e-6).abs() < 1e-12);
        assert_eq!(cfg.runtime.max_actions_per_turn, 10);
        assert!((cfg.runtime.reward_learning_cost - 0.0).abs() < f32::EPSILON);
        // Perception
        assert_eq!(cfg.runtime.perception.vision_radius, 5);
        // Mutation
        assert!((cfg.mutation.mutation_probability - 0.44).abs() < 1e-9);
        assert_eq!(cfg.mutation.per_birth_mutation_events_min, 1);
        assert_eq!(cfg.mutation.per_birth_mutation_events_max, 10);
        assert!((cfg.mutation.mesh_layer_probability - 0.2).abs() < 1e-9);
        assert_eq!(cfg.mutation.action_queue_cap, 4);
        // Complexity pressure
        assert_eq!(cfg.mutation.genome_size_cap, 1200);
        assert!(!cfg.mutation.genome_size_pressure_enabled);
        // Phenotype
        assert_eq!(cfg.mutation.phenotype.channel_step, 1);
        assert!((cfg.mutation.phenotype.channel_change_chance - 0.001).abs() < 1e-6);
        assert!((cfg.mutation.phenotype.polarity_flip_chance - 0.0002).abs() < 1e-6);
        // Reachable bias
        assert!((cfg.mutation.reachable_bias.topology - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.vm - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.graph - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.input_ref - 0.0).abs() < 1e-9);
        // Shared memory
        assert!((cfg.shared_memory.decay_rate - 0.0).abs() < f32::EPSILON);
        // Predation
        assert!((cfg.predation.steal_cost_rate - 0.05).abs() < 1e-6);
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.0).abs() < 1e-6);
        // Population
        assert_eq!(cfg.population.initial_creatures, 10000);
        assert_eq!(cfg.population.max_creatures, 100000);
        assert_eq!(cfg.population.founder_profile, FounderProfile::V3Alpha1);
    }

    #[test]
    fn founder_profile_serde_default_when_missing() {
        let json = r#"{"initial_creatures":2000,"max_creatures":100000}"#;
        let pop: PopulationConfig = serde_json::from_str(json).unwrap();
        assert_eq!(pop.founder_profile, FounderProfile::V3Alpha1);
    }

    #[test]
    fn founder_profile_serde_supports_tuned_variants() {
        let profiles = [
            ("forage_first_sparse", FounderProfile::ForageFirstSparse),
            (
                "forage_first_sparse_conservative",
                FounderProfile::ForageFirstSparseConservative,
            ),
            (
                "forage_first_sparse_rich_offspring",
                FounderProfile::ForageFirstSparseRichOffspring,
            ),
            (
                "forage_first_sparse_balanced",
                FounderProfile::ForageFirstSparseBalanced,
            ),
        ];

        for (wire, expected) in profiles {
            let json = format!(
                r#"{{"initial_creatures":2000,"max_creatures":100000,"founder_profile":"{wire}"}}"#
            );
            let pop: PopulationConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(pop.founder_profile, expected);
        }
    }

    #[test]
    fn normalize_nan_food_growth_rate_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.growth_rate = f32::NAN;
        cfg.normalize();
        assert!((cfg.world.food.growth_rate - 0.09).abs() < 1e-6);
    }

    #[test]
    fn normalize_clamps_growth_rate_above_one() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.growth_rate = 1.5;
        cfg.normalize();
        assert!((cfg.world.food.growth_rate - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_food_max_density_non_positive_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.max_density = 0.0;
        cfg.normalize();
        assert!((cfg.world.food.max_density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_initial_density_clamps_to_max_density() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.max_density = 0.8;
        cfg.world.food.initial_density = 1.5;
        cfg.normalize();
        assert!((cfg.world.food.initial_density - 0.8).abs() < 1e-6);
    }

    #[test]
    fn normalize_clamps_spread_density_ratio_above_one() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.spread_density_ratio = 1.5;
        cfg.normalize();
        assert!((cfg.world.food.spread_density_ratio - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_nan_spread_density_ratio_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.spread_density_ratio = f32::NAN;
        cfg.normalize();
        assert!((cfg.world.food.spread_density_ratio - 0.25).abs() < 1e-6);
    }

    #[test]
    fn normalize_negative_spread_density_ratio_clamps_to_zero() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.spread_density_ratio = -0.5;
        cfg.normalize();
        assert!((cfg.world.food.spread_density_ratio - 0.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_negative_max_energy_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.lifecycle.max_energy = -1.0;
        cfg.normalize();
        assert!((cfg.energy.lifecycle.max_energy - 200.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_min_reproduce_age_preserves_zero() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.lifecycle.min_reproduce_age = 0;
        cfg.normalize();
        assert_eq!(cfg.energy.lifecycle.min_reproduce_age, 0);
    }

    #[test]
    fn normalize_min_reproduce_age_keeps_configured_value() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.lifecycle.min_reproduce_age = 25;
        cfg.normalize();
        assert_eq!(cfg.energy.lifecycle.min_reproduce_age, 25);
    }

    #[test]
    fn normalize_zero_age_reference_ticks_falls_back_to_default() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.lifecycle.age_reference_ticks = 0;
        cfg.normalize();
        assert_eq!(cfg.energy.lifecycle.age_reference_ticks, 500);
    }

    #[test]
    fn normalize_age_reference_ticks_keeps_configured_value() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.lifecycle.age_reference_ticks = 1_000;
        cfg.normalize();
        assert_eq!(cfg.energy.lifecycle.age_reference_ticks, 1_000);
    }

    #[test]
    fn normalize_zero_max_mesh_hops_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.max_mesh_hops = 0;
        cfg.normalize();
        assert_eq!(cfg.runtime.max_mesh_hops, 1024);
    }

    #[test]
    fn normalize_nan_vm_opcode_cost_multiplier_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.vm.opcode_cost_multiplier = f32::NAN;
        cfg.normalize();
        assert!((cfg.runtime.vm.opcode_cost_multiplier - 1e-6).abs() < 1e-12);
    }

    #[test]
    fn normalize_invalid_vm_step_ramp_cost_falls_back() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0] {
            let mut cfg = SimulationConfig::default();
            cfg.runtime.vm.step_ramp_cost = invalid;
            cfg.normalize();
            assert!((cfg.runtime.vm.step_ramp_cost - 1e-6).abs() < 1e-12);
        }
    }

    #[test]
    fn normalize_keeps_valid_vm_step_ramp_settings() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.vm.step_ramp_cost = 0.0;
        cfg.runtime.vm.step_ramp_allowance = 0;
        cfg.normalize();
        assert_eq!(cfg.runtime.vm.step_ramp_cost, 0.0);
        assert_eq!(cfg.runtime.vm.step_ramp_allowance, 0);
    }

    #[test]
    fn vm_step_ramp_fields_default_when_absent_from_json() {
        let vm: VmRuntimeConfig =
            serde_json::from_str(r#"{"opcode_cost_multiplier": 0.5}"#).expect("vm config parses");
        assert_eq!(vm.step_ramp_allowance, 100);
        assert!((vm.step_ramp_cost - 1e-6).abs() < 1e-12);
    }

    #[test]
    fn normalize_mutation_events_max_below_min_corrects() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.per_birth_mutation_events_min = 3;
        cfg.mutation.per_birth_mutation_events_max = 1; // below min
        cfg.normalize();
        assert!(
            cfg.mutation.per_birth_mutation_events_max
                >= cfg.mutation.per_birth_mutation_events_min
        );
    }

    #[test]
    fn normalize_zero_action_queue_cap_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.action_queue_cap = 0;
        cfg.normalize();
        assert_eq!(cfg.mutation.action_queue_cap, 1);
    }

    #[test]
    fn normalize_huge_action_queue_cap_clamped() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.max_actions_per_turn = 100_000;
        cfg.mutation.action_queue_cap = 100_000;
        cfg.normalize();
        assert_eq!(cfg.mutation.action_queue_cap, 21845);
    }

    #[test]
    fn normalize_action_queue_cap_clamped_to_max_actions_per_turn() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.max_actions_per_turn = 3;
        cfg.mutation.action_queue_cap = 10;
        cfg.normalize();
        assert_eq!(cfg.mutation.action_queue_cap, 3);
    }

    #[test]
    fn normalize_zero_max_actions_per_turn_caps_action_queue_after_fallback() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.max_actions_per_turn = 0;
        cfg.mutation.action_queue_cap = 100_000;
        cfg.normalize();
        assert_eq!(cfg.runtime.max_actions_per_turn, 10);
        assert_eq!(cfg.mutation.action_queue_cap, 10);
    }

    #[test]
    fn normalize_vision_radius_zero_clamped_to_one() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.perception.vision_radius = 0;
        cfg.normalize();
        assert_eq!(cfg.runtime.perception.vision_radius, 1);
    }

    #[test]
    fn normalize_vision_radius_above_eight_clamped() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.perception.vision_radius = 20;
        cfg.normalize();
        assert_eq!(cfg.runtime.perception.vision_radius, 8);
    }

    #[test]
    fn normalize_phenotype_zero_channel_step_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.phenotype.channel_step = 0;
        cfg.normalize();
        assert_eq!(cfg.mutation.phenotype.channel_step, 1);
    }

    #[test]
    fn normalize_phenotype_polarity_flip_chance_clamped() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.phenotype.polarity_flip_chance = 1.5;
        cfg.normalize();
        assert!((cfg.mutation.phenotype.polarity_flip_chance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_phenotype_channel_change_chance_clamped() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.phenotype.channel_change_chance = 1.5;
        cfg.normalize();
        assert!((cfg.mutation.phenotype.channel_change_chance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_phenotype_channel_change_chance_negative_clamped() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.phenotype.channel_change_chance = -0.5;
        cfg.normalize();
        assert!((cfg.mutation.phenotype.channel_change_chance - 0.0).abs() < 1e-6);
    }

    #[test]
    fn config_serde_roundtrip_preserves_defaults() {
        let cfg = SimulationConfig::default();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        let cfg2: SimulationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.world.width, cfg2.world.width);
        assert_eq!(cfg.runtime.max_mesh_hops, cfg2.runtime.max_mesh_hops);
        assert!(
            (cfg.mutation.mutation_probability - cfg2.mutation.mutation_probability).abs() < 1e-12
        );
        assert!((cfg.predation.steal_cost_rate - cfg2.predation.steal_cost_rate).abs() < 1e-6);
    }

    #[test]
    fn config_default_failed_action_penalty() {
        let cfg = SimulationConfig::default();
        assert!((cfg.energy.costs.failed_action_penalty - 1.0).abs() < 1e-6);
    }

    #[test]
    fn config_normalize_failed_action_penalty_nan() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.costs.failed_action_penalty = f32::NAN;
        cfg.normalize();
        assert!((cfg.energy.costs.failed_action_penalty - 1.0).abs() < 1e-6);
    }

    #[test]
    fn config_normalize_failed_action_penalty_negative_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.costs.failed_action_penalty = -3.0;
        cfg.normalize();
        assert!((cfg.energy.costs.failed_action_penalty - 1.0).abs() < 1e-6);
    }

    #[test]
    fn deny_unknown_fields_rejects_extra_key() {
        let result = serde_json::from_str::<SimulationConfig>(r#"{"unknown_key": 1}"#);
        assert!(result.is_err(), "unknown key must be rejected");
    }

    // ── PredationConfig tests ──────────────────────────────────────────────

    #[test]
    fn predation_config_default_values() {
        let cfg = PredationConfig::default();
        assert!((cfg.steal_cost_rate - 0.05).abs() < 1e-6);
        assert!((cfg.kill_complexity_bonus_multiplier - 0.0).abs() < 1e-6);
    }

    #[test]
    fn simulation_config_has_predation_field() {
        let cfg = SimulationConfig::default();
        assert!((cfg.predation.steal_cost_rate - 0.05).abs() < 1e-6);
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_predation_steal_cost_rate_clamped_above_one() {
        let mut cfg = SimulationConfig::default();
        cfg.predation.steal_cost_rate = 1.5;
        cfg.normalize();
        assert!((cfg.predation.steal_cost_rate - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_predation_steal_cost_rate_clamped_negative() {
        let mut cfg = SimulationConfig::default();
        cfg.predation.steal_cost_rate = -0.5;
        cfg.normalize();
        assert!((cfg.predation.steal_cost_rate - 0.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_predation_steal_cost_rate_nan_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.predation.steal_cost_rate = f32::NAN;
        cfg.normalize();
        assert!((cfg.predation.steal_cost_rate - 0.05).abs() < 1e-6);
    }

    #[test]
    fn normalize_predation_kill_bonus_negative_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.predation.kill_complexity_bonus_multiplier = -1.0;
        cfg.normalize();
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_predation_kill_bonus_nan_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.predation.kill_complexity_bonus_multiplier = f32::NAN;
        cfg.normalize();
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_predation_kill_bonus_infinity_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.predation.kill_complexity_bonus_multiplier = f32::INFINITY;
        cfg.normalize();
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.0).abs() < 1e-6);
    }

    // ── SharedMemoryConfig tests ──────────────────────────────────────────

    #[test]
    fn shared_memory_config_default_decay_zero() {
        let cfg = SharedMemoryConfig::default();
        assert!((cfg.decay_rate - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_shared_memory_decay_rate_nan_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.shared_memory.decay_rate = f32::NAN;
        cfg.normalize();
        assert!((cfg.shared_memory.decay_rate - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_shared_memory_decay_rate_above_one_clamped() {
        let mut cfg = SimulationConfig::default();
        cfg.shared_memory.decay_rate = 1.5;
        cfg.normalize();
        assert!((cfg.shared_memory.decay_rate - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_shared_memory_decay_rate_negative_clamped() {
        let mut cfg = SimulationConfig::default();
        cfg.shared_memory.decay_rate = -0.5;
        cfg.normalize();
        assert!((cfg.shared_memory.decay_rate - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_nan_reward_learning_cost_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.reward_learning_cost = f32::NAN;
        cfg.normalize();
        assert!((cfg.runtime.reward_learning_cost - 0.0).abs() < f32::EPSILON);
    }

    // ── ComplexityEnergyCostConfig tests ───────────────────────────────────

    #[test]
    fn complexity_multiplier_below_threshold_returns_one() {
        let cc = ComplexityEnergyCostConfig::default();
        assert!((cc.multiplier(0) - 1.0).abs() < f32::EPSILON);
        assert!((cc.multiplier(30) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn complexity_multiplier_at_threshold_returns_one() {
        let cc = ComplexityEnergyCostConfig::default(); // threshold = 50
        assert!((cc.multiplier(50) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn complexity_multiplier_above_threshold_scales_linearly() {
        let cc = ComplexityEnergyCostConfig {
            enabled: true,
            ..ComplexityEnergyCostConfig::default()
        };
        // threshold=50, scaling=0.002
        // complexity 200: 1.0 + (200-50) * 0.002 = 1.3
        assert!((cc.multiplier(200) - 1.3).abs() < 1e-6);
        // complexity 550: 1.0 + (550-50) * 0.002 = 1.0 + 1.0 = 2.0
        assert!((cc.multiplier(550) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn complexity_multiplier_disabled_returns_one() {
        let cc = ComplexityEnergyCostConfig {
            enabled: false,
            threshold: 50,
            scaling_factor: 0.002,
        };
        assert!((cc.multiplier(1000) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn complexity_multiplier_threshold_zero_penalizes_all() {
        let cc = ComplexityEnergyCostConfig {
            enabled: true,
            threshold: 0,
            scaling_factor: 0.01,
        };
        // complexity 0: at threshold, returns 1.0
        assert!((cc.multiplier(0) - 1.0).abs() < f32::EPSILON);
        // complexity 100: 1.0 + 100 * 0.01 = 2.0
        assert!((cc.multiplier(100) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_complexity_cost_nan_scaling_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.complexity_cost.scaling_factor = f32::NAN;
        cfg.normalize();
        assert!((cfg.energy.complexity_cost.scaling_factor - 0.002).abs() < 1e-6);
    }

    #[test]
    fn normalize_complexity_cost_negative_scaling_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.complexity_cost.scaling_factor = -0.5;
        cfg.normalize();
        assert!((cfg.energy.complexity_cost.scaling_factor - 0.002).abs() < 1e-6);
    }

    // ── AgeEnergyCostConfig tests ─────────────────────────────────────────

    #[test]
    fn age_cost_config_default_values() {
        let ac = AgeEnergyCostConfig::default();
        assert!(ac.enabled);
        assert_eq!(ac.grace_ticks, 100);
        assert_eq!(ac.age_cap, 200);
        assert!((ac.max_multiplier - 10.0).abs() < 1e-6);
    }

    #[test]
    fn energy_config_has_age_cost_field() {
        let cfg = SimulationConfig::default();
        assert!(cfg.energy.age_cost.enabled);
        assert_eq!(cfg.energy.age_cost.grace_ticks, 100);
        assert_eq!(cfg.energy.age_cost.age_cap, 200);
        assert!((cfg.energy.age_cost.max_multiplier - 10.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_age_cost_max_multiplier_nan_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.age_cost.max_multiplier = f32::NAN;
        cfg.normalize();
        assert!((cfg.energy.age_cost.max_multiplier - 10.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_age_cost_max_multiplier_below_one_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.age_cost.max_multiplier = 0.5;
        cfg.normalize();
        assert!((cfg.energy.age_cost.max_multiplier - 10.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_age_cost_max_multiplier_infinity_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.age_cost.max_multiplier = f32::INFINITY;
        cfg.normalize();
        assert!((cfg.energy.age_cost.max_multiplier - 10.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_age_cost_max_multiplier_exactly_one_kept() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.age_cost.max_multiplier = 1.0;
        cfg.normalize();
        assert!((cfg.energy.age_cost.max_multiplier - 1.0).abs() < 1e-6);
    }

    #[test]
    fn age_cost_config_serde_roundtrip() {
        let cfg = SimulationConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let cfg2: SimulationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg2.energy.age_cost.grace_ticks, 100);
        assert_eq!(cfg2.energy.age_cost.age_cap, 200);
        assert!((cfg2.energy.age_cost.max_multiplier - 10.0).abs() < 1e-6);
    }

    #[test]
    fn age_cost_config_serde_defaults_grace_ticks_when_missing() {
        let json = r#"{"enabled":true,"age_cap":600,"max_multiplier":10.0}"#;
        let ac: AgeEnergyCostConfig = serde_json::from_str(json).unwrap();
        assert_eq!(ac.grace_ticks, 100);
        assert_eq!(ac.age_cap, 600);
    }

    #[test]
    fn age_cost_config_serde_default_when_missing() {
        // EnergyConfig without age_cost field should get defaults via #[serde(default)]
        let json = r#"{"lifecycle":{"initial_energy":20.0,"max_energy":200.0,"energy_decay_per_tick":0.5,"min_reproduce_energy":1.0,"default_offspring_energy":100.0},"costs":{"move_cost":1.0,"eat_cost":0.0,"eat_reward_per_food":5.0,"noop_cost":0.05,"reproduce_cost":0.1,"failed_action_penalty":5.0},"complexity_cost":{"enabled":true,"threshold":50,"scaling_factor":0.002}}"#;
        let ec: EnergyConfig = serde_json::from_str(json).unwrap();
        assert!(ec.age_cost.enabled);
        assert_eq!(ec.age_cost.grace_ticks, 100);
        assert_eq!(ec.age_cost.age_cap, 200);
    }

    #[test]
    fn energy_lifecycle_serde_defaults_min_reproduce_age_when_missing() {
        let json = r#"{"initial_energy":20.0,"max_energy":200.0,"energy_decay_per_tick":0.5,"min_reproduce_energy":30.0,"default_offspring_energy":100.0}"#;
        let lifecycle: EnergyLifecycleConfig = serde_json::from_str(json).unwrap();
        assert_eq!(lifecycle.min_reproduce_age, 20);
        assert_eq!(lifecycle.age_reference_ticks, 500);
    }

    #[test]
    fn energy_lifecycle_serde_roundtrip_preserves_min_reproduce_age() {
        let lifecycle = EnergyLifecycleConfig {
            min_reproduce_age: 42,
            ..EnergyLifecycleConfig::default()
        };
        let json = serde_json::to_string(&lifecycle).unwrap();
        let parsed: EnergyLifecycleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.min_reproduce_age, 42);
    }

    #[test]
    fn energy_lifecycle_serde_defaults_genome_carry_cost_when_missing() {
        let json = r#"{"initial_energy":20.0,"max_energy":200.0,"energy_decay_per_tick":0.5,"min_reproduce_energy":30.0,"default_offspring_energy":100.0}"#;
        let lifecycle: EnergyLifecycleConfig = serde_json::from_str(json).unwrap();
        assert_eq!(lifecycle.genome_carry_cost_per_unit, 1e-4);
    }

    #[test]
    fn energy_lifecycle_serde_roundtrip_preserves_genome_carry_cost() {
        let lifecycle = EnergyLifecycleConfig {
            genome_carry_cost_per_unit: 0.25,
            ..EnergyLifecycleConfig::default()
        };
        let json = serde_json::to_string(&lifecycle).unwrap();
        let parsed: EnergyLifecycleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.genome_carry_cost_per_unit, 0.25);
    }

    #[test]
    fn energy_lifecycle_serde_defaults_genome_replication_cost_when_missing() {
        let json = r#"{"initial_energy":20.0,"max_energy":200.0,"energy_decay_per_tick":0.5,"min_reproduce_energy":30.0,"default_offspring_energy":100.0}"#;
        let lifecycle: EnergyLifecycleConfig = serde_json::from_str(json).unwrap();
        assert_eq!(lifecycle.genome_replication_cost_per_unit, 0.1);
    }

    #[test]
    fn energy_lifecycle_serde_roundtrip_preserves_genome_replication_cost() {
        let lifecycle = EnergyLifecycleConfig {
            genome_replication_cost_per_unit: 0.25,
            ..EnergyLifecycleConfig::default()
        };
        let json = serde_json::to_string(&lifecycle).unwrap();
        let parsed: EnergyLifecycleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.genome_replication_cost_per_unit, 0.25);
    }

    #[test]
    fn normalize_invalid_genome_replication_cost_falls_back() {
        for bad in [f32::NAN, f32::INFINITY, -1.0] {
            let mut cfg = SimulationConfig::default();
            cfg.energy.lifecycle.genome_replication_cost_per_unit = bad;
            cfg.normalize();
            assert_eq!(cfg.energy.lifecycle.genome_replication_cost_per_unit, 0.1);
        }
    }

    #[test]
    fn normalize_genome_replication_cost_preserves_zero_and_configured_values() {
        for rate in [0.0, 0.25] {
            let mut cfg = SimulationConfig::default();
            cfg.energy.lifecycle.genome_replication_cost_per_unit = rate;
            cfg.normalize();
            assert_eq!(cfg.energy.lifecycle.genome_replication_cost_per_unit, rate);
        }
    }

    #[test]
    fn normalize_nan_genome_carry_cost_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.lifecycle.genome_carry_cost_per_unit = f32::NAN;
        cfg.normalize();
        assert_eq!(cfg.energy.lifecycle.genome_carry_cost_per_unit, 1e-4);
    }

    #[test]
    fn normalize_negative_genome_carry_cost_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.lifecycle.genome_carry_cost_per_unit = -1.0;
        cfg.normalize();
        assert_eq!(cfg.energy.lifecycle.genome_carry_cost_per_unit, 1e-4);
    }

    #[test]
    fn normalize_genome_carry_cost_preserves_zero_and_configured_values() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
        cfg.normalize();
        assert_eq!(cfg.energy.lifecycle.genome_carry_cost_per_unit, 0.0);

        cfg.energy.lifecycle.genome_carry_cost_per_unit = 0.5;
        cfg.normalize();
        assert_eq!(cfg.energy.lifecycle.genome_carry_cost_per_unit, 0.5);
    }

    #[test]
    fn age_multiplier_at_age_zero_returns_one() {
        let ac = AgeEnergyCostConfig::default();
        assert!((ac.multiplier(0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn age_multiplier_stays_one_through_grace_period() {
        let ac = AgeEnergyCostConfig::default();
        assert!((ac.multiplier(100) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn age_multiplier_ramps_quadratically_between_grace_and_cap() {
        let ac = AgeEnergyCostConfig::default();
        // age=150: progress=(150-100)/(200-100)=0.5; 1.0 + 9.0 * 0.25 = 3.25
        assert!((ac.multiplier(150) - 3.25).abs() < 1e-4);
    }

    #[test]
    fn age_multiplier_at_cap_returns_max() {
        let ac = AgeEnergyCostConfig::default();
        assert!((ac.multiplier(200) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn age_multiplier_above_cap_clamped() {
        let ac = AgeEnergyCostConfig::default();
        assert!((ac.multiplier(500) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn age_multiplier_disabled_returns_one() {
        let ac = AgeEnergyCostConfig {
            enabled: false,
            grace_ticks: 100,
            age_cap: 200,
            max_multiplier: 10.0,
        };
        assert!((ac.multiplier(400) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn age_multiplier_zero_cap_returns_one() {
        let ac = AgeEnergyCostConfig {
            enabled: true,
            grace_ticks: 100,
            age_cap: 0,
            max_multiplier: 10.0,
        };
        assert!((ac.multiplier(100) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn age_multiplier_reaches_max_after_grace_when_cap_is_not_later() {
        let ac = AgeEnergyCostConfig {
            enabled: true,
            grace_ticks: 100,
            age_cap: 50,
            max_multiplier: 10.0,
        };
        assert!((ac.multiplier(100) - 1.0).abs() < f32::EPSILON);
        assert!((ac.multiplier(101) - 10.0).abs() < f32::EPSILON);
    }

    proptest! {
        #[test]
        fn age_multiplier_is_bounded_and_monotonic(
            grace_ticks in 0u64..1_000,
            ramp_ticks in 1u64..1_000,
            first_age in 0u64..3_000,
            second_age in 0u64..3_000,
            max_multiplier in 1.0f32..1_000.0,
        ) {
            let ac = AgeEnergyCostConfig {
                enabled: true,
                grace_ticks,
                age_cap: grace_ticks + ramp_ticks,
                max_multiplier,
            };
            let younger = first_age.min(second_age);
            let older = first_age.max(second_age);
            let younger_multiplier = ac.multiplier(younger);
            let older_multiplier = ac.multiplier(older);

            prop_assert!(younger_multiplier >= 1.0);
            prop_assert!(older_multiplier <= max_multiplier);
            prop_assert!(younger_multiplier <= older_multiplier);
        }
    }

    // ── EnergyConfig::action_cost_multiplier tests ────────────────────────

    #[test]
    fn action_cost_multiplier_composes_complexity_and_age() {
        let mut ec = EnergyConfig::default();
        ec.complexity_cost.enabled = true;
        // complexity=200, age=150
        // complexity_mult = 1.0 + (200-50)*0.002 = 1.3
        // age_mult = 1.0 + 9.0 * 0.25 = 3.25
        // combined = 1.3 * 3.25 = 4.225
        let mult = ec.action_cost_multiplier(200, 150);
        assert!((mult - 4.225).abs() < 1e-3);
    }

    #[test]
    fn action_cost_multiplier_baseline_returns_one() {
        let ec = EnergyConfig::default();
        // complexity=0 (below threshold), age=0 → both return 1.0
        let mult = ec.action_cost_multiplier(0, 0);
        assert!((mult - 1.0).abs() < f32::EPSILON);
    }

    // ── EnergyConfig::adjusted_action_cost tests ───────────────────────────

    #[test]
    fn adjusted_action_cost_scales_base_cost_by_multiplier() {
        let ec = EnergyConfig::default();
        let base_cost = 5.0_f32;
        // complexity=200, age=250 → multiplier = 4.225 (see test above)
        let expected = base_cost * ec.action_cost_multiplier(200, 250);
        let result = ec.adjusted_action_cost(base_cost, 200, 250);
        assert!((result - expected).abs() < 1e-3);
    }

    #[test]
    fn adjusted_action_cost_baseline_returns_base_cost() {
        let ec = EnergyConfig::default();
        let base_cost = 3.0_f32;
        // complexity=0, age=0 → multiplier = 1.0
        let result = ec.adjusted_action_cost(base_cost, 0, 0);
        assert!((result - base_cost).abs() < f32::EPSILON);
    }

    #[test]
    fn adjusted_action_cost_zero_base_returns_zero() {
        let ec = EnergyConfig::default();
        // Even with high complexity/age, zero base cost stays zero.
        let result = ec.adjusted_action_cost(0.0, 200, 400);
        assert!((result - 0.0).abs() < f32::EPSILON);
    }

    // ── ReachableBiasConfig tests ─────────────────────────────────────────

    #[test]
    fn reachable_bias_config_defaults() {
        let rb = ReachableBiasConfig::default();
        assert!((rb.topology - 0.0).abs() < 1e-9);
        assert!((rb.vm - 0.0).abs() < 1e-9);
        assert!((rb.graph - 0.0).abs() < 1e-9);
        assert!((rb.input_ref - 0.0).abs() < 1e-9);
    }

    #[test]
    fn reachable_bias_config_on_mutation_config_defaults() {
        let cfg = SimulationConfig::default();
        assert!((cfg.mutation.reachable_bias.topology - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.vm - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.graph - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.input_ref - 0.0).abs() < 1e-9);
    }

    #[test]
    fn reachable_bias_serde_roundtrip() {
        let cfg = SimulationConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let cfg2: SimulationConfig = serde_json::from_str(&json).unwrap();
        assert!((cfg2.mutation.reachable_bias.topology - 0.0).abs() < 1e-9);
        assert!((cfg2.mutation.reachable_bias.input_ref - 0.0).abs() < 1e-9);
    }

    #[test]
    fn reachable_bias_serde_default_when_missing() {
        // MutationConfig JSON without reachable_bias should get defaults
        let json = serde_json::to_string(&MutationConfig::default()).unwrap();
        // Remove the reachable_bias field from the JSON
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let mut obj = v.as_object().unwrap().clone();
        obj.remove("reachable_bias");
        let stripped = serde_json::to_string(&obj).unwrap();
        let mc: MutationConfig = serde_json::from_str(&stripped).unwrap();
        assert!((mc.reachable_bias.topology - 0.0).abs() < 1e-9);
    }

    #[test]
    fn normalize_reachable_bias_clamps_above_one() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.reachable_bias.topology = 1.5;
        cfg.mutation.reachable_bias.vm = 2.0;
        cfg.normalize();
        assert!((cfg.mutation.reachable_bias.topology - 1.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.vm - 1.0).abs() < 1e-9);
    }

    #[test]
    fn normalize_reachable_bias_clamps_negative() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.reachable_bias.graph = -0.5;
        cfg.mutation.reachable_bias.input_ref = -1.0;
        cfg.normalize();
        assert!((cfg.mutation.reachable_bias.graph - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.input_ref - 0.0).abs() < 1e-9);
    }

    #[test]
    fn normalize_reachable_bias_nan_falls_back_to_zero() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.reachable_bias.topology = f64::NAN;
        cfg.mutation.reachable_bias.vm = f64::NAN;
        cfg.mutation.reachable_bias.graph = f64::INFINITY;
        cfg.mutation.reachable_bias.input_ref = f64::NEG_INFINITY;
        cfg.normalize();
        assert!((cfg.mutation.reachable_bias.topology - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.vm - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.graph - 0.0).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.input_ref - 0.0).abs() < 1e-9);
    }

    // ── Executed-bias targeting tests (T11.F17) ──────────────────────────

    #[test]
    fn executed_targeting_defaults() {
        let cfg = SimulationConfig::default();
        assert!((cfg.mutation.executed_bias - 0.9).abs() < 1e-9);
        assert_eq!(cfg.mutation.executed_window_ticks, 100);
    }

    #[test]
    fn large_copy_tuning_defaults_and_recipe_roundtrip() {
        let mut value = serde_json::to_value(MutationConfig::default()).unwrap();
        assert_eq!(value["large_copy_weight_percent"], 25);
        value
            .as_object_mut()
            .unwrap()
            .remove("large_copy_weight_percent");
        let config: MutationConfig = serde_json::from_value(value).unwrap();
        assert_eq!(config.large_copy_weight_percent, 25);
        assert!(!config.genome_size_pressure_enabled);
    }

    proptest! {
        #[test]
        fn large_copy_tuning_normalizes(percent in any::<u8>()) {
            let mut config = SimulationConfig::default();
            config.mutation.large_copy_weight_percent = percent;
            config.normalize();
            prop_assert_eq!(config.mutation.large_copy_weight_percent, percent.min(100));
            let roundtrip: SimulationConfig = serde_json::from_str(
                &serde_json::to_string(&config).unwrap()).unwrap();
            prop_assert_eq!(roundtrip.mutation.large_copy_weight_percent, percent.min(100));
        }
    }

    #[test]
    fn executed_targeting_serde_defaults_when_missing() {
        let json = serde_json::to_string(&MutationConfig::default()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let mut obj = value.as_object().unwrap().clone();
        obj.remove("executed_bias");
        obj.remove("executed_window_ticks");
        let stripped = serde_json::to_string(&obj).unwrap();
        let mc: MutationConfig = serde_json::from_str(&stripped).unwrap();
        assert!((mc.executed_bias - 0.9).abs() < 1e-9);
        assert_eq!(mc.executed_window_ticks, 100);
    }

    #[test]
    fn normalize_executed_bias_clamps_and_replaces_non_finite() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.executed_bias = 1.5;
        cfg.normalize();
        assert!((cfg.mutation.executed_bias - 1.0).abs() < 1e-9);
        cfg.mutation.executed_bias = -0.5;
        cfg.normalize();
        assert!((cfg.mutation.executed_bias - 0.0).abs() < 1e-9);
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            cfg.mutation.executed_bias = value;
            cfg.normalize();
            assert!((cfg.mutation.executed_bias - 0.9).abs() < 1e-9);
        }
    }

    #[test]
    fn normalize_executed_window_zero_falls_back_to_the_default() {
        let mut cfg = SimulationConfig::default();
        cfg.mutation.executed_window_ticks = 0;
        cfg.normalize();
        assert_eq!(cfg.mutation.executed_window_ticks, 100);
        cfg.mutation.executed_window_ticks = 7;
        cfg.normalize();
        assert_eq!(cfg.mutation.executed_window_ticks, 7);
    }

    #[test]
    fn startup_failed_action_penalty_ramp_defaults_to_enabled() {
        let cfg = SimulationConfig::default();
        assert!(cfg.startup.ramps.failed_action_penalty.enabled);
        assert!((cfg.startup.ramps.failed_action_penalty.start - 0.0).abs() < 1e-6);
        assert!((cfg.startup.ramps.failed_action_penalty.end - 1.0).abs() < 1e-6);
        assert_eq!(cfg.startup.ramps.failed_action_penalty.target_tick, 62680);
        assert!((cfg.failed_action_penalty_for_tick(0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn startup_failed_action_penalty_ramp_interpolates_until_target_tick() {
        let mut cfg = SimulationConfig::default();
        cfg.startup.ramps.failed_action_penalty.enabled = true;
        cfg.startup.ramps.failed_action_penalty.start = 5.0;
        cfg.startup.ramps.failed_action_penalty.end = 30.0;
        cfg.startup.ramps.failed_action_penalty.target_tick = 1000;
        cfg.apply_startup_overrides();

        assert!((cfg.energy.costs.failed_action_penalty - 30.0).abs() < 1e-6);
        assert!((cfg.failed_action_penalty_for_tick(0) - 5.0).abs() < 1e-6);
        assert!((cfg.failed_action_penalty_for_tick(500) - 17.5).abs() < 1e-5);
        assert!((cfg.failed_action_penalty_for_tick(1000) - 30.0).abs() < 1e-6);
        assert!((cfg.failed_action_penalty_for_tick(2000) - 30.0).abs() < 1e-6);
    }

    #[test]
    fn startup_failed_action_penalty_ramp_active_only_before_target_tick() {
        let mut cfg = SimulationConfig::default();
        cfg.startup.ramps.failed_action_penalty.enabled = true;
        cfg.startup.ramps.failed_action_penalty.target_tick = 3;

        assert!(cfg.failed_action_penalty_ramp_active(0));
        assert!(cfg.failed_action_penalty_ramp_active(2));
        assert!(!cfg.failed_action_penalty_ramp_active(3));
    }

    // ── Food config / FertilityConfig / AnnealingConfig tests ─────────────

    #[test]
    fn food_config_default_has_primary_type_and_all_food_fertility_layers() {
        let config = FoodConfig::default();
        assert!(config.shared.occupancy_depletion.enabled);
        assert!((config.shared.occupancy_depletion.deposit_per_occupied_tick - 0.08).abs() < 1e-6);
        assert_eq!(config.types.len(), 1);
        assert_eq!(config.types[0].name, "Primary Food");
        assert!(config.fertility.enabled);
        assert_eq!(config.fertility.min_fertility, 0.0);
        assert_eq!(config.fertility.max_fertility, 2.0);
        assert_eq!(config.fertility.layers.len(), 2);
        for layer in &config.fertility.layers {
            match &layer.algorithm {
                FertilityAlgorithm::PoissonBlobs {
                    blob_count,
                    min_radius,
                    max_radius,
                    falloff,
                    ..
                } => {
                    assert_eq!(*blob_count, 900);
                    assert!((*min_radius - 5.0).abs() < f32::EPSILON);
                    assert!((*max_radius - 15.0).abs() < f32::EPSILON);
                    assert!((*falloff - 0.5).abs() < f32::EPSILON);
                }
                other => panic!("expected PoissonBlobs layer, got {other:?}"),
            }
            assert!(matches!(layer.target, FertilityLayerTarget::AllFoods));
        }
        assert!(!config.shared.annealing.enabled);
        assert_eq!(config.shared.annealing.ramp_ticks, 5000);
    }

    #[test]
    fn food_config_deserializes_with_defaults_when_omitted() {
        let json = r#"{"shared":{"growth_rate":0.09,"initial_density":1.0,"initial_coverage":0.54,"spread_threshold_ratio":0.8,"spread_density_ratio":0.25,"recovery_spawn_rate":0.01,"recovery_floor_ratio":0.01,"max_density":1.0}}"#;
        let config: FoodConfig = serde_json::from_str(json).unwrap();
        assert!(config.shared.occupancy_depletion.enabled);
        assert!(config.fertility.enabled);
        assert!(!config.shared.annealing.enabled);
        assert_eq!(config.types.len(), 1);
        assert_eq!(config.fertility.layers.len(), 2);
        assert!(matches!(
            config.fertility.layers[0].target,
            FertilityLayerTarget::AllFoods
        ));
    }

    #[test]
    fn fertility_algorithm_serializes_as_tagged_enum() {
        let layer = FertilityLayer::default();
        let json = serde_json::to_string(&layer).unwrap();
        assert!(json.contains("PoissonBlobs"));
    }

    #[test]
    fn normalize_nan_occupancy_depletion_deposit_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.occupancy_depletion.deposit_per_occupied_tick = f32::NAN;
        cfg.normalize();
        assert!((cfg.world.food.occupancy_depletion.deposit_per_occupied_tick - 0.08).abs() < 1e-6);
    }

    #[test]
    fn grazing_defaults_are_the_production_values() {
        let cfg = SimulationConfig::default();
        let grazing = &cfg.world.food.grazing;
        assert!(grazing.enabled);
        assert!((grazing.factor - 0.5).abs() < 1e-6);
        assert!((grazing.floor - 0.05).abs() < 1e-6);
        assert_eq!(grazing.recovery_ticks, 1000);
    }

    #[test]
    fn grazing_block_and_its_fields_are_serde_defaulted() {
        let without_block: FoodResourceConfig = serde_json::from_str(
            r#"{"growth_rate":0.1,"initial_density":1.0,"initial_coverage":0.5,
                "spread_threshold_ratio":0.8,"spread_density_ratio":0.25,
                "recovery_spawn_rate":0.01,"recovery_floor_ratio":0.01,"max_density":1.0}"#,
        )
        .unwrap();
        assert!(without_block.grazing.enabled);
        assert_eq!(without_block.grazing.recovery_ticks, 1000);

        let partial: GrazingConfig = serde_json::from_str(r#"{"factor":0.25}"#).unwrap();
        assert!(partial.enabled);
        assert!((partial.factor - 0.25).abs() < 1e-6);
        assert!((partial.floor - 0.05).abs() < 1e-6);
        assert_eq!(partial.recovery_ticks, 1000);
    }

    #[test]
    fn normalize_grazing_clamps_ratios_and_restores_defaults() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.grazing.factor = 1.5;
        cfg.world.food.grazing.floor = -0.5;
        cfg.world.food.grazing.recovery_ticks = 0;
        cfg.normalize();
        assert!((cfg.world.food.grazing.factor - 1.0).abs() < 1e-6);
        assert!((cfg.world.food.grazing.floor - 0.0).abs() < 1e-6);
        assert_eq!(cfg.world.food.grazing.recovery_ticks, 1000);

        cfg.world.food.grazing.factor = f32::NAN;
        cfg.world.food.grazing.floor = f32::INFINITY;
        cfg.world.food.grazing.recovery_ticks = 1;
        cfg.normalize();
        assert!((cfg.world.food.grazing.factor - 0.5).abs() < 1e-6);
        assert!((cfg.world.food.grazing.floor - 0.05).abs() < 1e-6);
        assert_eq!(cfg.world.food.grazing.recovery_ticks, 1);

        // `floor <= factor` is not required.
        cfg.world.food.grazing.factor = 0.2;
        cfg.world.food.grazing.floor = 0.9;
        cfg.normalize();
        assert!((cfg.world.food.grazing.factor - 0.2).abs() < 1e-6);
        assert!((cfg.world.food.grazing.floor - 0.9).abs() < 1e-6);
    }

    #[test]
    fn normalize_grazing_caps_recovery_ticks_at_the_property_domain() {
        let mut cfg = SimulationConfig::default();
        for (input, expected) in [(10_000, 10_000), (10_001, 10_000), (u32::MAX, 10_000)] {
            cfg.world.food.grazing.recovery_ticks = input;
            cfg.normalize();
            assert_eq!(
                cfg.world.food.grazing.recovery_ticks, expected,
                "input {input}"
            );
        }
    }

    #[test]
    fn normalize_occupancy_depletion_deposit_clamps_to_unit_interval() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.occupancy_depletion.deposit_per_occupied_tick = 1.5;
        cfg.normalize();
        assert!((cfg.world.food.occupancy_depletion.deposit_per_occupied_tick - 1.0).abs() < 1e-6);

        cfg.world.food.occupancy_depletion.deposit_per_occupied_tick = -0.5;
        cfg.normalize();
        assert!((cfg.world.food.occupancy_depletion.deposit_per_occupied_tick - 0.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_world_food_restores_dimensions_and_default_type() {
        let mut world = WorldConfig {
            width: 0,
            height: 0,
            ..WorldConfig::default()
        };
        world.food.types.clear();

        normalize_world_food(&mut world);

        assert_eq!(world.width, 1600);
        assert_eq!(world.height, 1600);
        assert_eq!(world.food.types.len(), 1);
        assert_eq!(world.food.types, FoodConfig::default().types);
    }

    #[test]
    fn normalize_food_config_clamps_type_density_to_shared_max_density() {
        let mut food = FoodConfig::default();
        food.shared.max_density = 0.5;
        food.types[0].initial_density = 2.0;

        normalize_food_config(&mut food);

        assert!((food.types[0].initial_density - 0.5).abs() < 1e-6);
        assert!((food.shared.initial_density - 0.5).abs() < 1e-6);
    }

    #[test]
    fn normalize_food_config_clamps_growth_inhibitor_to_unit_interval() {
        let mut food = FoodConfig {
            types: vec![
                FoodTypeConfig {
                    growth_inhibitor: -0.3,
                    ..FoodTypeConfig::default()
                },
                FoodTypeConfig {
                    growth_inhibitor: 2.4,
                    ..FoodTypeConfig::default()
                },
            ],
            ..FoodConfig::default()
        };

        normalize_food_config(&mut food);

        assert_eq!(food.types[0].growth_inhibitor, 0.0);
        assert_eq!(food.types[1].growth_inhibitor, 1.0);
    }

    #[test]
    fn food_type_config_defaults_growth_inhibitor_when_field_is_omitted() {
        let result: Result<FoodTypeConfig, _> = serde_json::from_value(serde_json::json!({
            "name": "Legacy Food",
            "color": "#ffffff",
            "initial_density": 0.4,
            "initial_coverage": 0.6
        }));
        assert_eq!(result.unwrap().growth_inhibitor, 0.2);
    }

    #[test]
    fn normalize_invalid_fertility_target_falls_back_to_all_foods() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.types = vec![FoodTypeConfig::default()];
        cfg.world.food.fertility.layers[0].target = FertilityLayerTarget::SingleType {
            type_idx: OrdinaryFoodTypeId::new(99),
        };
        cfg.normalize();
        assert!(matches!(
            cfg.world.food.fertility.layers[0].target,
            FertilityLayerTarget::AllFoods
        ));
    }

    #[test]
    fn normalize_syncs_primary_food_type_density_and_coverage_into_shared_fields() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.types[0].initial_density = 0.72;
        cfg.world.food.types[0].initial_coverage = 0.31;

        cfg.normalize();

        assert!((cfg.world.food.shared.initial_density - 0.72).abs() < 1e-6);
        assert!((cfg.world.food.shared.initial_coverage - 0.31).abs() < 1e-6);
    }

    #[test]
    fn normalize_syncs_top_level_fertility_and_annealing_into_shared_runtime_config() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.fertility.enabled = false;
        cfg.world.food.annealing.enabled = true;
        cfg.world.food.annealing.ramp_ticks = 1234;

        cfg.normalize();

        assert!(!cfg.world.food.shared.fertility.enabled);
        assert!(cfg.world.food.shared.annealing.enabled);
        assert_eq!(cfg.world.food.shared.annealing.ramp_ticks, 1234);
    }

    #[test]
    fn single_type_preserves_top_level_fertility_and_annealing() {
        let mut shared = FoodResourceConfig::default();
        shared.fertility.enabled = false;
        shared.fertility.min_fertility = 0.25;
        shared.fertility.max_fertility = 1.25;
        shared.annealing.enabled = true;
        shared.annealing.ramp_ticks = 777;

        let config = FoodConfig::single_type(shared.clone());

        assert_eq!(config.fertility.enabled, shared.fertility.enabled);
        assert!((config.fertility.min_fertility - shared.fertility.min_fertility).abs() < 1e-6);
        assert!((config.fertility.max_fertility - shared.fertility.max_fertility).abs() < 1e-6);
        assert_eq!(config.annealing.enabled, shared.annealing.enabled);
        assert_eq!(config.annealing.ramp_ticks, shared.annealing.ramp_ticks);
    }

    #[test]
    fn single_type_syncs_primary_type_density_and_coverage() {
        let shared = FoodResourceConfig {
            initial_density: 0.77,
            initial_coverage: 0.33,
            ..FoodResourceConfig::default()
        };

        let config = FoodConfig::single_type(shared.clone());

        assert_eq!(config.types.len(), 1);
        assert!((config.types[0].initial_density - shared.initial_density).abs() < 1e-6);
        assert!((config.types[0].initial_coverage - shared.initial_coverage).abs() < 1e-6);
    }

    /// Guards the failure mode recorded in the 2026-09-07 config panel apply
    /// audit: when the shipped default is not a fixed point of `normalize`,
    /// every runtime config PATCH is rejected against a freshly started server.
    #[test]
    fn normalize_leaves_the_default_config_unchanged() {
        let expected = serde_json::to_value(SimulationConfig::default()).unwrap();
        let mut config = SimulationConfig::default();

        config.normalize();

        assert_eq!(serde_json::to_value(&config).unwrap(), expected);
    }

    /// Draws every field with a cross-field normalization rule plus a sample of
    /// the clamped floats, including non-finite values.
    fn perturbed_config_strategy() -> impl Strategy<Value = SimulationConfig> {
        let clamped_float = prop_oneof![
            Just(f32::NAN),
            Just(f32::INFINITY),
            Just(f32::NEG_INFINITY),
            -5.0f32..5.0f32,
        ];
        (
            (0u32..20_000, 0u32..20_000),
            (0usize..24, 0usize..24),
            (0u32..24, 0u32..24),
            0usize..8,
            (clamped_float.clone(), clamped_float.clone(), clamped_float),
            prop_oneof![Just(f64::NAN), -0.5f64..1.5f64],
            prop_oneof![Just(f64::NAN), -0.5f64..1.5f64],
        )
            .prop_map(
                |(
                    (initial_creatures, max_creatures),
                    (max_actions_per_turn, action_queue_cap),
                    (events_min, events_max),
                    action_log_capacity,
                    (max_density, initial_density, initial_coverage),
                    mutation_probability,
                    per_unit_rate,
                )| {
                    let mut config = SimulationConfig::default();
                    config.mutation.per_unit_rate = per_unit_rate;
                    config.population.initial_creatures = initial_creatures;
                    config.population.max_creatures = max_creatures;
                    config.runtime.max_actions_per_turn = max_actions_per_turn;
                    config.mutation.action_queue_cap = action_queue_cap;
                    config.mutation.per_birth_mutation_events_min = events_min;
                    config.mutation.per_birth_mutation_events_max = events_max;
                    config.action_log.capacity = action_log_capacity;
                    config.mutation.mutation_probability = mutation_probability;
                    config.world.food.shared.max_density = max_density;
                    config.world.food.shared.initial_density = initial_density;
                    config.world.food.shared.initial_coverage = initial_coverage;
                    for food_type in &mut config.world.food.types {
                        food_type.initial_density = initial_density;
                        food_type.initial_coverage = initial_coverage;
                    }
                    config
                },
            )
    }

    proptest! {
        /// `normalize` must reach a fixed point in one pass: the PATCH endpoint
        /// compares a merged config against its single normalization, so a
        /// second pass that still moves values would reject canonical configs.
        #[test]
        fn normalize_is_idempotent(config in perturbed_config_strategy()) {
            let mut once = config;
            once.normalize();
            let mut twice = once.clone();
            twice.normalize();

            prop_assert_eq!(
                serde_json::to_value(&twice).unwrap(),
                serde_json::to_value(&once).unwrap()
            );
        }
    }
}
