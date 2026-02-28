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
            default_offspring_energy: 8.0,
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

/// Combined energy config.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyConfig {
    pub lifecycle: EnergyLifecycleConfig,
    pub costs: EnergyCostsConfig,
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
    /// Energy cost per Hebbian weight update. Default 0.0 (free during initial rollout).
    #[serde(default)]
    pub hebbian_update_cost: f32,
    pub vm: VmRuntimeConfig,
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
            hebbian_update_cost: 0.0,
            vm: VmRuntimeConfig::default(),
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
    /// Genome complexity score above which pressure escalates against structural growth.
    pub complexity_cap: u32,
    /// Whether the complexity pressure gate is active.
    pub complexity_pressure_enabled: bool,
    pub phenotype: PhenotypeConfig,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            mutation_probability: 0.303,
            per_birth_mutation_events_min: 1,
            per_birth_mutation_events_max: 10,
            mesh_layer_probability: 0.2,
            complexity_cap: 1200,
            complexity_pressure_enabled: true,
            phenotype: PhenotypeConfig::default(),
        }
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
        el.default_offspring_energy = normalize_f32_finite_nonneg(el.default_offspring_energy, 8.0);

        let ec = &mut self.energy.costs;
        ec.move_cost = normalize_f32_finite_nonneg(ec.move_cost, 1.0);
        ec.eat_cost = normalize_f32_finite_nonneg(ec.eat_cost, 0.0);
        ec.noop_cost = normalize_f32_finite_nonneg(ec.noop_cost, 0.05);
        ec.reproduce_cost = normalize_f32_finite_nonneg(ec.reproduce_cost, 0.1);
        ec.eat_reward_per_food = normalize_f32_finite_nonneg(ec.eat_reward_per_food, 12.0);
        ec.failed_action_penalty = normalize_f32_finite_nonneg(ec.failed_action_penalty, 5.0);

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
        rt.hebbian_update_cost = normalize_f32_finite_nonneg(rt.hebbian_update_cost, 0.0);
        rt.vm.opcode_cost_multiplier =
            normalize_f32_finite_nonneg(rt.vm.opcode_cost_multiplier, 1e-6);

        let m = &mut self.mutation;
        m.mutation_probability = m.mutation_probability.clamp(0.0, 1.0);
        m.mesh_layer_probability = m.mesh_layer_probability.clamp(0.0, 1.0);
        if m.per_birth_mutation_events_min < 1 {
            m.per_birth_mutation_events_min = 1;
        }
        if m.per_birth_mutation_events_max < m.per_birth_mutation_events_min {
            m.per_birth_mutation_events_max = m.per_birth_mutation_events_min;
        }
        // complexity_cap: 0 disables pressure (handled by is_restricted), no normalization needed.
        // complexity_pressure_enabled: bool, no normalization needed.
        let ph = &mut m.phenotype;
        if ph.channel_step == 0 {
            ph.channel_step = 1;
        }
        ph.channel_change_chance = ph.channel_change_chance.clamp(0.0, 1.0);
        ph.polarity_flip_chance = ph.polarity_flip_chance.clamp(0.0, 1.0);

        let pred = &mut self.predation;
        pred.steal_cost_rate = normalize_f32_clamp(pred.steal_cost_rate, 0.0, 1.0, 0.2);
        pred.kill_complexity_bonus_multiplier =
            normalize_f32_finite_nonneg(pred.kill_complexity_bonus_multiplier, 0.05);

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
        assert!((cfg.energy.lifecycle.default_offspring_energy - 8.0).abs() < 1e-6);
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
        // Mutation
        assert!((cfg.mutation.mutation_probability - 0.303).abs() < 1e-9);
        assert_eq!(cfg.mutation.per_birth_mutation_events_min, 1);
        assert_eq!(cfg.mutation.per_birth_mutation_events_max, 10);
        assert!((cfg.mutation.mesh_layer_probability - 0.2).abs() < 1e-9);
        // Complexity pressure
        assert_eq!(cfg.mutation.complexity_cap, 1200);
        assert!(cfg.mutation.complexity_pressure_enabled);
        // Phenotype
        assert_eq!(cfg.mutation.phenotype.channel_step, 1);
        assert!((cfg.mutation.phenotype.channel_change_chance - 0.001).abs() < 1e-6);
        assert!((cfg.mutation.phenotype.polarity_flip_chance - 0.0002).abs() < 1e-6);
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
}
