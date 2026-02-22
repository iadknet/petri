/// Edge mode for the world grid.
/// Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum WorldEdgeMode {
    #[default]
    Wrap,
    Bounded,
}

/// Food substrate config. Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldFoodConfig {
    pub growth_rate: f32,
    pub initial_density: u8,
    pub initial_coverage: f32,
}

impl Default for WorldFoodConfig {
    fn default() -> Self {
        Self {
            growth_rate: 0.02,
            initial_density: 80,
            initial_coverage: 0.3,
        }
    }
}

/// World/grid config. Canonical owner: v3-world-grid-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
            max_energy: 100.0,
            energy_decay_per_tick: 0.2,
            min_reproduce_energy: 24.0,
            default_offspring_energy: 20.0,
        }
    }
}

/// Energy action costs config. Canonical owner: v3-runtime-config-spec.md Section 4.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnergyCostsConfig {
    pub move_cost: f32,
    pub eat_cost: f32,
    pub noop_cost: f32,
    pub reproduce_cost: f32,
    pub eat_reward_per_food: f32,
}

impl Default for EnergyCostsConfig {
    fn default() -> Self {
        Self {
            move_cost: 0.2,
            eat_cost: 0.0,
            noop_cost: 0.0,
            reproduce_cost: 2.0,
            eat_reward_per_food: 1.0,
        }
    }
}

/// Combined energy config.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EnergyConfig {
    pub lifecycle: EnergyLifecycleConfig,
    pub costs: EnergyCostsConfig,
}

/// VM runtime config. Canonical owner: v3-runtime-config-spec.md Section 2.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VmRuntimeConfig {
    pub opcode_cost_multiplier: f32,
}

impl Default for VmRuntimeConfig {
    fn default() -> Self {
        Self {
            opcode_cost_multiplier: 1.0,
        }
    }
}

/// Mesh/VM/graph execution limits. Canonical owner: v3-runtime-config-spec.md Section 2.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuntimeConfig {
    pub max_mesh_hops: u32,
    pub max_vm_steps: u32,
    pub max_graph_relax_iters: u32,
    pub graph_convergence_epsilon: f32,
    pub graph_convergence_stable_passes: u32,
    pub graph_node_base_cost: f32,
    pub vm: VmRuntimeConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_mesh_hops: 128,
            max_vm_steps: 1024,
            max_graph_relax_iters: 4,
            graph_convergence_epsilon: 1e-3,
            graph_convergence_stable_passes: 1,
            graph_node_base_cost: 1.0,
            vm: VmRuntimeConfig::default(),
        }
    }
}

/// Mutation tuning config. Canonical owner: v3-runtime-config-spec.md Section 3.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MutationConfig {
    pub mutation_probability: f64,
    pub per_birth_mutation_events_min: u32,
    pub per_birth_mutation_events_max: u32,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            mutation_probability: 0.01,
            per_birth_mutation_events_min: 1,
            per_birth_mutation_events_max: 4,
        }
    }
}

/// Population caps. Canonical owner: v3-runtime-config-spec.md Section 5.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PopulationConfig {
    pub initial_creatures: u32,
    pub max_creatures: u32,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            initial_creatures: 50,
            max_creatures: 1000,
        }
    }
}

/// Full simulation configuration.
/// Use `SimulationConfig::default()` to get canonical spec defaults.
/// Call `normalize()` after deserialization or programmatic construction to
/// apply fallback rules for out-of-range values.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SimulationConfig {
    pub world: WorldConfig,
    pub energy: EnergyConfig,
    pub runtime: RuntimeConfig,
    pub mutation: MutationConfig,
    pub population: PopulationConfig,
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
        w.food.growth_rate = normalize_f32_clamp(w.food.growth_rate, 0.0, 1.0, 0.02);
        w.food.initial_coverage = normalize_f32_clamp(w.food.initial_coverage, 0.0, 1.0, 0.3);

        let el = &mut self.energy.lifecycle;
        el.initial_energy = normalize_f32_finite_nonneg(el.initial_energy, 20.0);
        el.max_energy = normalize_f32_finite_min(el.max_energy, 1.0, 100.0);
        el.energy_decay_per_tick = normalize_f32_finite_nonneg(el.energy_decay_per_tick, 0.2);
        el.min_reproduce_energy = normalize_f32_finite_nonneg(el.min_reproduce_energy, 24.0);
        el.default_offspring_energy =
            normalize_f32_finite_nonneg(el.default_offspring_energy, 20.0);

        let ec = &mut self.energy.costs;
        ec.move_cost = normalize_f32_finite_nonneg(ec.move_cost, 0.2);
        ec.eat_cost = normalize_f32_finite_nonneg(ec.eat_cost, 0.0);
        ec.noop_cost = normalize_f32_finite_nonneg(ec.noop_cost, 0.0);
        ec.reproduce_cost = normalize_f32_finite_nonneg(ec.reproduce_cost, 2.0);
        ec.eat_reward_per_food = normalize_f32_finite_nonneg(ec.eat_reward_per_food, 1.0);

        let rt = &mut self.runtime;
        if rt.max_mesh_hops < 1 {
            rt.max_mesh_hops = 128;
        }
        if rt.max_vm_steps < 1 {
            rt.max_vm_steps = 1024;
        }
        if rt.max_graph_relax_iters < 1 {
            rt.max_graph_relax_iters = 4;
        }
        rt.graph_convergence_epsilon = normalize_f32_nonneg(rt.graph_convergence_epsilon, 1e-3);
        if rt.graph_convergence_stable_passes < 1 {
            rt.graph_convergence_stable_passes = 1;
        }
        rt.graph_node_base_cost = normalize_f32_nonneg(rt.graph_node_base_cost, 1.0);
        rt.vm.opcode_cost_multiplier =
            normalize_f32_finite_nonneg(rt.vm.opcode_cost_multiplier, 1.0);

        let m = &mut self.mutation;
        m.mutation_probability = m.mutation_probability.clamp(0.0, 1.0);
        if m.per_birth_mutation_events_min < 1 {
            m.per_birth_mutation_events_min = 1;
        }
        if m.per_birth_mutation_events_max < m.per_birth_mutation_events_min {
            m.per_birth_mutation_events_max = m.per_birth_mutation_events_min;
        }

        let p = &mut self.population;
        if p.initial_creatures < 1 {
            p.initial_creatures = 50;
        }
        if p.max_creatures < p.initial_creatures {
            p.max_creatures = 1000;
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
        assert!((cfg.world.food.growth_rate - 0.02).abs() < 1e-6);
        assert_eq!(cfg.world.food.initial_density, 80);
        assert!((cfg.world.food.initial_coverage - 0.3).abs() < 1e-6);
        // Energy lifecycle
        assert!((cfg.energy.lifecycle.initial_energy - 20.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.max_energy - 100.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.energy_decay_per_tick - 0.2).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.min_reproduce_energy - 24.0).abs() < 1e-6);
        assert!((cfg.energy.lifecycle.default_offspring_energy - 20.0).abs() < 1e-6);
        // Energy costs
        assert!((cfg.energy.costs.move_cost - 0.2).abs() < 1e-6);
        assert!((cfg.energy.costs.eat_cost - 0.0).abs() < 1e-6);
        assert!((cfg.energy.costs.noop_cost - 0.0).abs() < 1e-6);
        assert!((cfg.energy.costs.reproduce_cost - 2.0).abs() < 1e-6);
        assert!((cfg.energy.costs.eat_reward_per_food - 1.0).abs() < 1e-6);
        // Runtime
        assert_eq!(cfg.runtime.max_mesh_hops, 128);
        assert_eq!(cfg.runtime.max_vm_steps, 1024);
        assert_eq!(cfg.runtime.max_graph_relax_iters, 4);
        assert!((cfg.runtime.graph_convergence_epsilon - 1e-3).abs() < 1e-6);
        assert_eq!(cfg.runtime.graph_convergence_stable_passes, 1);
        assert!((cfg.runtime.graph_node_base_cost - 1.0).abs() < 1e-6);
        assert!((cfg.runtime.vm.opcode_cost_multiplier - 1.0).abs() < 1e-6);
        // Mutation
        assert!((cfg.mutation.mutation_probability - 0.01).abs() < 1e-9);
        assert_eq!(cfg.mutation.per_birth_mutation_events_min, 1);
        assert_eq!(cfg.mutation.per_birth_mutation_events_max, 4);
        // Population
        assert_eq!(cfg.population.initial_creatures, 50);
        assert_eq!(cfg.population.max_creatures, 1000);
    }

    #[test]
    fn normalize_nan_food_growth_rate_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.growth_rate = f32::NAN;
        cfg.normalize();
        assert!((cfg.world.food.growth_rate - 0.02).abs() < 1e-6);
    }

    #[test]
    fn normalize_clamps_growth_rate_above_one() {
        let mut cfg = SimulationConfig::default();
        cfg.world.food.growth_rate = 1.5;
        cfg.normalize();
        assert!((cfg.world.food.growth_rate - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_negative_max_energy_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.energy.lifecycle.max_energy = -1.0;
        cfg.normalize();
        assert!((cfg.energy.lifecycle.max_energy - 100.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_zero_max_mesh_hops_falls_back() {
        let mut cfg = SimulationConfig::default();
        cfg.runtime.max_mesh_hops = 0;
        cfg.normalize();
        assert_eq!(cfg.runtime.max_mesh_hops, 128);
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
    fn config_serde_roundtrip_preserves_defaults() {
        let cfg = SimulationConfig::default();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        let cfg2: SimulationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.world.width, cfg2.world.width);
        assert_eq!(cfg.runtime.max_mesh_hops, cfg2.runtime.max_mesh_hops);
        assert!(
            (cfg.mutation.mutation_probability - cfg2.mutation.mutation_probability).abs() < 1e-12
        );
    }
}
