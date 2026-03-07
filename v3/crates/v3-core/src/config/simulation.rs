/// Edge mode for the world grid.
/// Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum WorldEdgeMode {
    #[default]
    Wrap,
    Bounded,
}

/// Food substrate config. Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldFoodConfig {
    pub growth_rate: f32,
    pub initial_density: f32,
    pub initial_coverage: f32,
    pub spread_threshold_ratio: f32,
    pub spread_density_ratio: f32,
    pub recovery_spawn_rate: f32,
    pub recovery_floor_ratio: f32,
    pub max_density: f32,
}

impl Default for WorldFoodConfig {
    fn default() -> Self {
        Self {
            growth_rate: 0.096,
            initial_density: 1.0,
            initial_coverage: 0.15,
            spread_threshold_ratio: 0.8,
            spread_density_ratio: 0.25,
            recovery_spawn_rate: 0.01,
            recovery_floor_ratio: 0.01,
            max_density: 1.0,
        }
    }
}

/// World/grid config. Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldConfig {
    pub width: u16,
    pub height: u16,
    pub edge_mode: WorldEdgeMode,
    pub food: WorldFoodConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 400,
            height: 400,
            edge_mode: WorldEdgeMode::default(),
            food: WorldFoodConfig::default(),
        }
    }
}

/// Energy lifecycle config. Canonical owner: v3-runtime-config-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyLifecycleConfig {
    pub initial_energy: f32,
    pub max_energy: f32,
    pub energy_decay_per_tick: f32,
    pub min_reproduce_energy: f32,
    pub default_offspring_energy: f32,
}

impl Default for EnergyLifecycleConfig {
    fn default() -> Self {
        Self {
            initial_energy: 20.0,
            max_energy: 200.0,
            energy_decay_per_tick: 0.5,
            min_reproduce_energy: 1.0,
            default_offspring_energy: 100.0,
        }
    }
}

/// Energy action costs config. Canonical owner: v3-runtime-config-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyCostsConfig {
    pub move_cost: f32,
    pub eat_cost: f32,
    pub noop_cost: f32,
    pub reproduce_cost: f32,
    pub eat_reward_per_food: f32,
    /// Additional energy penalty applied when an action fails (move blocked, eat empty cell, etc.).
    pub failed_action_penalty: f32,
}

impl Default for EnergyCostsConfig {
    fn default() -> Self {
        Self {
            move_cost: 1.0,
            eat_cost: 0.0,
            noop_cost: 0.05,
            reproduce_cost: 0.1,
            eat_reward_per_food: 12.0,
            failed_action_penalty: 5.0,
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
            enabled: true,
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
    /// Age (in ticks) at which the maximum multiplier applies.
    pub age_cap: u64,
    /// Maximum energy cost multiplier at or beyond age_cap.
    pub max_multiplier: f32,
}

impl Default for AgeEnergyCostConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            age_cap: 500,
            max_multiplier: 10.0,
        }
    }
}

impl AgeEnergyCostConfig {
    /// Returns the energy cost multiplier for a creature of the given age.
    ///
    /// Formula: `1.0 + (max_multiplier - 1.0) * min(1.0, age / age_cap)^2`
    /// Returns 1.0 (no penalty) when disabled or age_cap is 0.
    #[inline]
    #[must_use]
    pub fn multiplier(&self, age: u64) -> f32 {
        if !self.enabled || self.age_cap == 0 {
            return 1.0;
        }
        let ratio = (age as f32 / self.age_cap as f32).min(1.0);
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

/// VM runtime config. Canonical owner: v3-runtime-config-spec.md Section 2.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VmRuntimeConfig {
    pub opcode_cost_multiplier: f32,
}

impl Default for VmRuntimeConfig {
    fn default() -> Self {
        Self {
            opcode_cost_multiplier: 1e-6,
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
    pub max_graph_relax_iters: u32,
    pub graph_convergence_epsilon: f32,
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
            topology: 0.7,
            vm: 0.7,
            graph: 0.7,
            input_ref: 0.5,
        }
    }
}

/// Mutation tuning config. Canonical owner: v3-runtime-config-spec.md Section 3.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationConfig {
    pub mutation_probability: f64,
    pub per_birth_mutation_events_min: u32,
    pub per_birth_mutation_events_max: u32,
    /// Probability of selecting the mesh (Topology) layer per mutation event.
    /// Complement (1 - this) selects the node-internal layer (VM/Graph/InputRef).
    pub mesh_layer_probability: f64,
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
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            mutation_probability: 0.303,
            per_birth_mutation_events_min: 1,
            per_birth_mutation_events_max: 10,
            mesh_layer_probability: 0.2,
            genome_size_cap: 1200,
            genome_size_pressure_enabled: true,
            action_queue_cap: 4,
            phenotype: PhenotypeConfig::default(),
            reachable_bias: ReachableBiasConfig::default(),
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
            steal_cost_rate: 0.2,
            kill_complexity_bonus_multiplier: 0.05,
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
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            initial_creatures: 2000,
            max_creatures: 100000,
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
    /// Apply normalization/fallback for out-of-range values per spec constraints.
    pub fn normalize(&mut self) {
        let w = &mut self.world;
        if w.width == 0 {
            w.width = 400;
        }
        if w.height == 0 {
            w.height = 400;
        }
        w.food.max_density = normalize_f32_finite_positive(w.food.max_density, 1.0);
        w.food.growth_rate = normalize_f32_clamp(w.food.growth_rate, 0.0, 1.0, 0.096);
        w.food.initial_coverage = normalize_f32_clamp(w.food.initial_coverage, 0.0, 1.0, 0.15);
        w.food.spread_threshold_ratio =
            normalize_f32_clamp(w.food.spread_threshold_ratio, 0.0, 1.0, 0.8);
        w.food.spread_density_ratio =
            normalize_f32_clamp(w.food.spread_density_ratio, 0.0, 1.0, 0.25);
        w.food.recovery_spawn_rate =
            normalize_f32_clamp(w.food.recovery_spawn_rate, 0.0, 1.0, 0.01);
        w.food.recovery_floor_ratio =
            normalize_f32_clamp(w.food.recovery_floor_ratio, 0.0, 1.0, 0.01);
        w.food.initial_density = normalize_f32_clamp(
            w.food.initial_density,
            0.0,
            w.food.max_density,
            w.food.max_density,
        );

        let el = &mut self.energy.lifecycle;
        el.initial_energy = normalize_f32_finite_nonneg(el.initial_energy, 20.0);
        el.max_energy = normalize_f32_finite_min(el.max_energy, 1.0, 200.0);
        el.energy_decay_per_tick = normalize_f32_finite_nonneg(el.energy_decay_per_tick, 0.5);
        el.min_reproduce_energy = normalize_f32_finite_nonneg(el.min_reproduce_energy, 1.0);
        el.default_offspring_energy =
            normalize_f32_finite_nonneg(el.default_offspring_energy, 100.0);

        let ec = &mut self.energy.costs;
        ec.move_cost = normalize_f32_finite_nonneg(ec.move_cost, 1.0);
        ec.eat_cost = normalize_f32_finite_nonneg(ec.eat_cost, 0.0);
        ec.noop_cost = normalize_f32_finite_nonneg(ec.noop_cost, 0.05);
        ec.reproduce_cost = normalize_f32_finite_nonneg(ec.reproduce_cost, 0.1);
        ec.eat_reward_per_food = normalize_f32_finite_nonneg(ec.eat_reward_per_food, 12.0);
        ec.failed_action_penalty = normalize_f32_finite_nonneg(ec.failed_action_penalty, 5.0);

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
        rt.perception.vision_radius = rt.perception.vision_radius.clamp(1, 8);

        let m = &mut self.mutation;
        m.mutation_probability = m.mutation_probability.clamp(0.0, 1.0);
        m.mesh_layer_probability = m.mesh_layer_probability.clamp(0.0, 1.0);
        if m.per_birth_mutation_events_min < 1 {
            m.per_birth_mutation_events_min = 1;
        }
        if m.per_birth_mutation_events_max < m.per_birth_mutation_events_min {
            m.per_birth_mutation_events_max = m.per_birth_mutation_events_min;
        }
        // Cap must be >= 1; upper bound prevents u16 overflow in sub_value_count.
        m.action_queue_cap = m.action_queue_cap.clamp(1, 21845);
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

        if self.action_log.capacity < 1 {
            self.action_log.capacity = 500;
        }

        let pred = &mut self.predation;
        pred.steal_cost_rate = normalize_f32_clamp(pred.steal_cost_rate, 0.0, 1.0, 0.2);
        pred.kill_complexity_bonus_multiplier =
            normalize_f32_finite_nonneg(pred.kill_complexity_bonus_multiplier, 0.05);

        self.shared_memory.decay_rate =
            normalize_f32_clamp(self.shared_memory.decay_rate, 0.0, 1.0, 0.0);

        let p = &mut self.population;
        if p.initial_creatures < 1 {
            p.initial_creatures = 2000;
        }
        if p.max_creatures < p.initial_creatures {
            p.max_creatures = 100000;
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

    #[test]
    fn default_config_matches_spec() {
        let cfg = SimulationConfig::default();
        // World
        assert_eq!(cfg.world.width, 400);
        assert_eq!(cfg.world.height, 400);
        assert!(matches!(cfg.world.edge_mode, WorldEdgeMode::Wrap));
        assert!((cfg.world.food.growth_rate - 0.096).abs() < 1e-6);
        assert!((cfg.world.food.initial_density - 1.0).abs() < 1e-6);
        assert!((cfg.world.food.initial_coverage - 0.15).abs() < 1e-6);
        assert!((cfg.world.food.spread_threshold_ratio - 0.8).abs() < 1e-6);
        assert!((cfg.world.food.spread_density_ratio - 0.25).abs() < 1e-6);
        assert!((cfg.world.food.recovery_spawn_rate - 0.01).abs() < 1e-6);
        assert!((cfg.world.food.recovery_floor_ratio - 0.01).abs() < 1e-6);
        assert!((cfg.world.food.max_density - 1.0).abs() < 1e-6);
        // Energy lifecycle
        assert!((cfg.energy.lifecycle.initial_energy - 20.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.max_energy - 200.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.energy_decay_per_tick - 0.5).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.min_reproduce_energy - 1.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.default_offspring_energy - 100.0).abs() < 1e-6);
        // Complexity energy cost
        assert!(cfg.energy.complexity_cost.enabled);
        assert_eq!(cfg.energy.complexity_cost.threshold, 50);
        assert!((cfg.energy.complexity_cost.scaling_factor - 0.002).abs() < 1e-6);
        // Age energy cost
        assert!(cfg.energy.age_cost.enabled);
        assert_eq!(cfg.energy.age_cost.age_cap, 500);
        assert!((cfg.energy.age_cost.max_multiplier - 10.0).abs() < 1e-6);
        // Energy costs
        assert!((cfg.energy.costs.move_cost - 1.0).abs() < 1e-6);
        assert!((cfg.energy.costs.eat_cost - 0.0).abs() < 1e-6);
        assert!((cfg.energy.costs.noop_cost - 0.05).abs() < 1e-6);
        assert!((cfg.energy.costs.reproduce_cost - 0.1).abs() < 1e-6);
        assert!((cfg.energy.costs.eat_reward_per_food - 12.0).abs() < 1e-6);
        assert!((cfg.energy.costs.failed_action_penalty - 5.0).abs() < 1e-6);
        // Runtime
        assert_eq!(cfg.runtime.max_mesh_hops, 1024);
        assert_eq!(cfg.runtime.max_vm_steps, 10000);
        assert_eq!(cfg.runtime.max_graph_relax_iters, 15);
        assert!((cfg.runtime.graph_convergence_epsilon - 1e-3).abs() < 1e-6);
        assert_eq!(cfg.runtime.graph_convergence_stable_passes, 2);
        assert!((cfg.runtime.graph_node_base_cost - 1e-5).abs() < 1e-9);
        assert!((cfg.runtime.vm.opcode_cost_multiplier - 1e-6).abs() < 1e-12);
        assert_eq!(cfg.runtime.max_actions_per_turn, 10);
        assert!((cfg.runtime.reward_learning_cost - 0.0).abs() < f32::EPSILON);
        // Perception
        assert_eq!(cfg.runtime.perception.vision_radius, 5);
        // Mutation
        assert!((cfg.mutation.mutation_probability - 0.303).abs() < 1e-9);
        assert_eq!(cfg.mutation.per_birth_mutation_events_min, 1);
        assert_eq!(cfg.mutation.per_birth_mutation_events_max, 10);
        assert!((cfg.mutation.mesh_layer_probability - 0.2).abs() < 1e-9);
        assert_eq!(cfg.mutation.action_queue_cap, 4);
        // Complexity pressure
        assert_eq!(cfg.mutation.genome_size_cap, 1200);
        assert!(cfg.mutation.genome_size_pressure_enabled);
        // Phenotype
        assert_eq!(cfg.mutation.phenotype.channel_step, 1);
        assert!((cfg.mutation.phenotype.channel_change_chance - 0.001).abs() < 1e-6);
        assert!((cfg.mutation.phenotype.polarity_flip_chance - 0.0002).abs() < 1e-6);
        // Reachable bias
        assert!((cfg.mutation.reachable_bias.topology - 0.7).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.vm - 0.7).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.graph - 0.7).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.input_ref - 0.5).abs() < 1e-9);
        // Shared memory
        assert!((cfg.shared_memory.decay_rate - 0.0).abs() < f32::EPSILON);
        // Predation
        assert!((cfg.predation.steal_cost_rate - 0.2).abs() < 1e-6);
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.05).abs() < 1e-6);
        // Population
        assert_eq!(cfg.population.initial_creatures, 2000);
        assert_eq!(cfg.population.max_creatures, 100000);
    }

    #[test]
    fn normalize_nan_food_growth_rate_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.growth_rate = f32::NAN;
        cfg.normalize();
        assert!((cfg.world.food.growth_rate - 0.096).abs() < 1e-6);
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
        cfg.mutation.action_queue_cap = 100_000;
        cfg.normalize();
        assert_eq!(cfg.mutation.action_queue_cap, 21845);
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
        assert!((cfg.energy.costs.failed_action_penalty - 5.0).abs() < 1e-6);
    }

    #[test]
    fn config_normalize_failed_action_penalty_nan() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.costs.failed_action_penalty = f32::NAN;
        cfg.normalize();
        assert!((cfg.energy.costs.failed_action_penalty - 5.0).abs() < 1e-6);
    }

    #[test]
    fn config_normalize_failed_action_penalty_negative_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.costs.failed_action_penalty = -3.0;
        cfg.normalize();
        assert!((cfg.energy.costs.failed_action_penalty - 5.0).abs() < 1e-6);
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
        assert!((cfg.steal_cost_rate - 0.2).abs() < 1e-6);
        assert!((cfg.kill_complexity_bonus_multiplier - 0.05).abs() < 1e-6);
    }

    #[test]
    fn simulation_config_has_predation_field() {
        let cfg = SimulationConfig::default();
        assert!((cfg.predation.steal_cost_rate - 0.2).abs() < 1e-6);
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.05).abs() < 1e-6);
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
        assert!((cfg.predation.steal_cost_rate - 0.2).abs() < 1e-6);
    }

    #[test]
    fn normalize_predation_kill_bonus_negative_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.predation.kill_complexity_bonus_multiplier = -1.0;
        cfg.normalize();
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.05).abs() < 1e-6);
    }

    #[test]
    fn normalize_predation_kill_bonus_nan_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.predation.kill_complexity_bonus_multiplier = f32::NAN;
        cfg.normalize();
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.05).abs() < 1e-6);
    }

    #[test]
    fn normalize_predation_kill_bonus_infinity_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.predation.kill_complexity_bonus_multiplier = f32::INFINITY;
        cfg.normalize();
        assert!((cfg.predation.kill_complexity_bonus_multiplier - 0.05).abs() < 1e-6);
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
        let cc = ComplexityEnergyCostConfig::default();
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
        assert_eq!(ac.age_cap, 500);
        assert!((ac.max_multiplier - 10.0).abs() < 1e-6);
    }

    #[test]
    fn energy_config_has_age_cost_field() {
        let cfg = SimulationConfig::default();
        assert!(cfg.energy.age_cost.enabled);
        assert_eq!(cfg.energy.age_cost.age_cap, 500);
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
        assert_eq!(cfg2.energy.age_cost.age_cap, 500);
        assert!((cfg2.energy.age_cost.max_multiplier - 10.0).abs() < 1e-6);
    }

    #[test]
    fn age_cost_config_serde_default_when_missing() {
        // EnergyConfig without age_cost field should get defaults via #[serde(default)]
        let json = r#"{"lifecycle":{"initial_energy":20.0,"max_energy":200.0,"energy_decay_per_tick":0.5,"min_reproduce_energy":1.0,"default_offspring_energy":100.0},"costs":{"move_cost":1.0,"eat_cost":0.0,"noop_cost":0.05,"reproduce_cost":0.1,"eat_reward_per_food":12.0,"failed_action_penalty":5.0},"complexity_cost":{"enabled":true,"threshold":50,"scaling_factor":0.002}}"#;
        let ec: EnergyConfig = serde_json::from_str(json).unwrap();
        assert!(ec.age_cost.enabled);
        assert_eq!(ec.age_cost.age_cap, 500);
    }

    #[test]
    fn age_multiplier_at_age_zero_returns_one() {
        let ac = AgeEnergyCostConfig::default();
        assert!((ac.multiplier(0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn age_multiplier_mid_range_quadratic() {
        let ac = AgeEnergyCostConfig::default(); // age_cap=500, max_multiplier=10.0
                                                 // age=250: ratio=0.5, 1.0 + 9.0 * 0.25 = 3.25
        assert!((ac.multiplier(250) - 3.25).abs() < 1e-4);
    }

    #[test]
    fn age_multiplier_at_cap_returns_max() {
        let ac = AgeEnergyCostConfig::default();
        assert!((ac.multiplier(500) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn age_multiplier_above_cap_clamped() {
        let ac = AgeEnergyCostConfig::default();
        assert!((ac.multiplier(1000) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn age_multiplier_disabled_returns_one() {
        let ac = AgeEnergyCostConfig {
            enabled: false,
            age_cap: 500,
            max_multiplier: 10.0,
        };
        assert!((ac.multiplier(400) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn age_multiplier_zero_cap_returns_one() {
        let ac = AgeEnergyCostConfig {
            enabled: true,
            age_cap: 0,
            max_multiplier: 10.0,
        };
        assert!((ac.multiplier(100) - 1.0).abs() < f32::EPSILON);
    }

    // ── EnergyConfig::action_cost_multiplier tests ────────────────────────

    #[test]
    fn action_cost_multiplier_composes_complexity_and_age() {
        let ec = EnergyConfig::default();
        // complexity=200, age=250
        // complexity_mult = 1.0 + (200-50)*0.002 = 1.3
        // age_mult = 1.0 + 9.0 * 0.25 = 3.25
        // combined = 1.3 * 3.25 = 4.225
        let mult = ec.action_cost_multiplier(200, 250);
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
        assert!((rb.topology - 0.7).abs() < 1e-9);
        assert!((rb.vm - 0.7).abs() < 1e-9);
        assert!((rb.graph - 0.7).abs() < 1e-9);
        assert!((rb.input_ref - 0.5).abs() < 1e-9);
    }

    #[test]
    fn reachable_bias_config_on_mutation_config_defaults() {
        let cfg = SimulationConfig::default();
        assert!((cfg.mutation.reachable_bias.topology - 0.7).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.vm - 0.7).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.graph - 0.7).abs() < 1e-9);
        assert!((cfg.mutation.reachable_bias.input_ref - 0.5).abs() < 1e-9);
    }

    #[test]
    fn reachable_bias_serde_roundtrip() {
        let cfg = SimulationConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let cfg2: SimulationConfig = serde_json::from_str(&json).unwrap();
        assert!((cfg2.mutation.reachable_bias.topology - 0.7).abs() < 1e-9);
        assert!((cfg2.mutation.reachable_bias.input_ref - 0.5).abs() < 1e-9);
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
        assert!((mc.reachable_bias.topology - 0.7).abs() < 1e-9);
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
}
