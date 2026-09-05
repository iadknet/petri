//! Battery `neighborhood-v1`: the fixed sensor scenarios and genome-execution
//! signature that every mutational-neighborhood reading (T11.F01) is taken
//! against.
//!
//! A signature is a property of the genome alone: every execution starts from
//! zeroed shared memory, zeroed previous memory, and fresh graph runtime
//! state, so two genomes are compared on identical footing regardless of any
//! world trajectory. The battery has two parts, executed in this fixed order:
//! 48 single-tick snapshots (seed 7), then 8 sequences of 4 ticks each (seed
//! 8) in which shared memory and graph state persist across the ticks of one
//! sequence under the production per-tick bookkeeping
//! ([`crate::simulation::advance_shared_memory`]).

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::config::RuntimeConfig;
use crate::contracts::WorldAction;
use crate::creature::genome::CreatureGenome;
use crate::creature::state::GraphRuntimeState;
use crate::runtime::mesh::execute_creature_mesh_with_reserve;
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;
use crate::simulation::advance_shared_memory;

/// Number of single-tick snapshot scenarios in the battery.
pub const SNAPSHOT_COUNT: usize = 48;
/// Number of multi-tick sequences in the battery.
pub const SEQUENCE_COUNT: usize = 8;
/// Ticks per multi-tick sequence.
pub const SEQUENCE_LEN: usize = 4;
/// Fixed seed for the snapshot scenarios.
pub const SNAPSHOT_SEED: u64 = 7;
/// Fixed seed for the sequence scenarios.
pub const SEQUENCE_SEED: u64 = 8;

const AGE_CHOICES: [f32; 6] = [0.0, 5.0, 19.0, 20.0, 50.0, 200.0];
const ENERGY_CHOICES: [f32; 6] = [5.0, 15.0, 25.0, 31.0, 45.0, 80.0];
const RESERVE_CHOICES: [f32; 4] = [0.0, 2.0, 4.0, 8.0];

/// One battery scenario, drawn once at battery generation and immutable
/// afterward: the assembled sensor snapshot (built once here rather than on
/// every genome's execution, since it is fixed for the whole battery) and the
/// per-tick creature state a genome is evaluated against.
#[derive(Debug, Clone, PartialEq)]
struct Scenario {
    sensors: SensorSnapshot,
    energy: f32,
    reserve: f32,
}

/// `Some(x)` with probability `1 - p_zero`, else `0.0`, matching Appendix A's
/// generator so the audit's founder readings stay reproducible under this
/// indicator.
fn nonzero_or_zero(rng: &mut SmallRng, p_zero: f64) -> f32 {
    if rng.gen_bool(p_zero) {
        0.0
    } else {
        rng.gen_range(0.1f32..=1.0)
    }
}

fn draw_scenario(rng: &mut SmallRng, food_type_count: usize) -> Scenario {
    let food_here_by_type: Vec<f32> = (0..food_type_count)
        .map(|type_idx| nonzero_or_zero(rng, if type_idx == 0 { 0.5 } else { 0.6 }))
        .collect();
    let age = AGE_CHOICES[rng.gen_range(0..AGE_CHOICES.len())];
    let energy = ENERGY_CHOICES[rng.gen_range(0..ENERGY_CHOICES.len())];
    let reserve = RESERVE_CHOICES[rng.gen_range(0..RESERVE_CHOICES.len())];
    let neighbor_food_by_type: Vec<[f32; 8]> = (0..food_type_count)
        .map(|_| std::array::from_fn(|_| nonzero_or_zero(rng, 0.6)))
        .collect();
    let neighbor_occupied: [f32; 8] =
        std::array::from_fn(|_| if rng.gen_bool(0.2) { 1.0 } else { 0.0 });

    let food_here = food_here_by_type.first().copied().unwrap_or(0.0);
    let neighbor_food = neighbor_food_by_type.first().copied().unwrap_or([0.0; 8]);
    let sensors = SensorSnapshot {
        local: StaticInputs {
            food_here,
            neighbor_food,
            neighbor_barrier: [0.0; 8],
            neighbor_occupied,
            generation: 0.0,
            age_ticks: age,
        },
        typed_local_food: TypedFoodLocalSnapshot {
            food_here_by_type,
            neighbor_food_by_type,
        },
        perception: PerceptionSnapshot::zeroed(food_type_count),
    };
    Scenario {
        sensors,
        energy,
        reserve,
    }
}

fn draw_scenarios(seed: u64, count: usize, food_type_count: usize) -> Vec<Scenario> {
    let mut rng = SmallRng::seed_from_u64(seed);
    (0..count)
        .map(|_| draw_scenario(&mut rng, food_type_count))
        .collect()
}

/// The complete `neighborhood-v1` battery: the fixed snapshot and sequence
/// scenarios, generated once and shared across every subject evaluated in a
/// report run.
#[derive(Debug, Clone, PartialEq)]
pub struct Battery {
    snapshots: Vec<Scenario>,
    /// `SEQUENCE_COUNT` chunks of `SEQUENCE_LEN` scenarios each.
    sequences: Vec<Vec<Scenario>>,
}

impl Battery {
    /// Generate the fixed `neighborhood-v1` battery for a world configured
    /// with `food_type_count` ordinary food types.
    #[must_use]
    pub fn generate(food_type_count: usize) -> Self {
        let snapshots = draw_scenarios(SNAPSHOT_SEED, SNAPSHOT_COUNT, food_type_count);
        let sequence_scenarios = draw_scenarios(
            SEQUENCE_SEED,
            SEQUENCE_COUNT * SEQUENCE_LEN,
            food_type_count,
        );
        let sequences = sequence_scenarios
            .chunks(SEQUENCE_LEN)
            .map(<[Scenario]>::to_vec)
            .collect();
        Self {
            snapshots,
            sequences,
        }
    }

    /// Evaluate `genome`'s complete execution signature against this battery.
    /// `shared_memory_decay_rate` is the production
    /// `SimulationConfig::shared_memory.decay_rate` that governs bookkeeping
    /// between the ticks of a sequence.
    #[must_use]
    pub fn signature(
        &self,
        genome: &CreatureGenome,
        runtime: &RuntimeConfig,
        shared_memory_decay_rate: f32,
    ) -> Signature {
        let snapshots = self
            .snapshots
            .iter()
            .map(|scenario| self.execute_single_tick(genome, scenario, runtime))
            .collect();
        let sequences = self
            .sequences
            .iter()
            .map(|sequence| {
                self.execute_sequence(genome, sequence, runtime, shared_memory_decay_rate)
            })
            .collect();
        Signature {
            snapshots,
            sequences,
        }
    }

    fn execute_single_tick(
        &self,
        genome: &CreatureGenome,
        scenario: &Scenario,
        runtime: &RuntimeConfig,
    ) -> Vec<WorldAction> {
        let mut energy = scenario.energy;
        let mut shared_memory = [0.0f32; 16];
        let prev_shared_memory = [0.0f32; 16];
        let mut graph_runtime = GraphRuntimeState::new();
        execute_creature_mesh_with_reserve(
            genome,
            &scenario.sensors,
            &mut energy,
            scenario.reserve,
            &mut shared_memory,
            &prev_shared_memory,
            &mut graph_runtime,
            runtime,
        )
        .actions
    }

    /// Execute one sequence of ticks, applying the production shared-memory
    /// bookkeeping between ticks (a fresh snapshot-then-decay before each
    /// tick's cognition, mirroring `run_phase_0`'s order within a tick).
    /// Energy and reserve reset to each tick's scenario values; shared memory
    /// and graph state persist across the sequence.
    fn execute_sequence(
        &self,
        genome: &CreatureGenome,
        sequence: &[Scenario],
        runtime: &RuntimeConfig,
        decay_rate: f32,
    ) -> Vec<Vec<WorldAction>> {
        let mut shared_memory = [0.0f32; 16];
        let mut prev_shared_memory = [0.0f32; 16];
        let mut graph_runtime = GraphRuntimeState::new();
        sequence
            .iter()
            .map(|scenario| {
                advance_shared_memory(&mut shared_memory, &mut prev_shared_memory, decay_rate);
                let mut energy = scenario.energy;
                execute_creature_mesh_with_reserve(
                    genome,
                    &scenario.sensors,
                    &mut energy,
                    scenario.reserve,
                    &mut shared_memory,
                    &prev_shared_memory,
                    &mut graph_runtime,
                    runtime,
                )
                .actions
            })
            .collect()
    }
}

/// One genome's complete signature against a [`Battery`]: the 48 single-tick
/// snapshot outputs, then the 8 sequences of 4 chained-tick outputs.
#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub snapshots: Vec<Vec<WorldAction>>,
    pub sequences: Vec<Vec<Vec<WorldAction>>>,
}

impl Signature {
    /// Total number of executions this signature carries (48 + 32 at the
    /// predeclared battery sizes).
    #[must_use]
    pub fn execution_count(&self) -> usize {
        self.snapshots.len() + self.sequences.iter().map(Vec::len).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FounderProfile, RuntimeConfig};
    use crate::creature::founder::founder_genome;

    #[test]
    fn generate_is_deterministic_for_a_fixed_food_type_count() {
        assert_eq!(Battery::generate(2), Battery::generate(2));
    }

    #[test]
    fn snapshot_and_sequence_sizes_match_the_predeclared_battery() {
        let battery = Battery::generate(2);
        assert_eq!(battery.snapshots.len(), SNAPSHOT_COUNT);
        assert_eq!(battery.sequences.len(), SEQUENCE_COUNT);
        for sequence in &battery.sequences {
            assert_eq!(sequence.len(), SEQUENCE_LEN);
        }
    }

    #[test]
    fn signature_execution_count_matches_the_battery_size() {
        let battery = Battery::generate(2);
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let signature = battery.signature(&founder, &RuntimeConfig::default(), 0.0);
        assert_eq!(
            signature.execution_count(),
            SNAPSHOT_COUNT + SEQUENCE_COUNT * SEQUENCE_LEN
        );
    }

    #[test]
    fn signature_is_a_pure_function_of_the_genome_and_battery() {
        let battery = Battery::generate(2);
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let runtime = RuntimeConfig::default();
        let a = battery.signature(&founder, &runtime, 0.0);
        let b = battery.signature(&founder, &runtime, 0.0);
        assert_eq!(
            a, b,
            "the same genome against the same battery always yields the same signature"
        );
    }

    #[test]
    fn execute_single_tick_never_returns_an_empty_action_queue() {
        // `into_actions_or_noop()` guarantees a NoOp fallback, so a fresh
        // subject's single-tick action queue is never empty; a stub that
        // always returns `vec![]` would fail this on every scenario.
        let battery = Battery::generate(2);
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let runtime = RuntimeConfig::default();
        for scenario in &battery.snapshots {
            let actions = battery.execute_single_tick(&founder, scenario, &runtime);
            assert!(!actions.is_empty());
        }
    }

    #[test]
    fn nonzero_or_zero_is_always_zero_when_p_zero_is_one() {
        let mut rng = SmallRng::seed_from_u64(3);
        for _ in 0..10 {
            assert_eq!(nonzero_or_zero(&mut rng, 1.0), 0.0);
        }
    }

    #[test]
    fn nonzero_or_zero_is_in_range_when_p_zero_is_zero() {
        let mut rng = SmallRng::seed_from_u64(3);
        for _ in 0..50 {
            let value = nonzero_or_zero(&mut rng, 0.0);
            assert!((0.1..=1.0).contains(&value), "{value}");
        }
    }

    /// `draw_scenario` gives the first food type a lower zero-probability
    /// (`0.5`) than every other type (`0.6`), per Appendix A's generator.
    /// Swapping the `type_idx == 0` test would swap which type gets which
    /// probability; a large fixed-seed sample distinguishes the two by its
    /// per-type zero fraction (SE ~0.008 at 4,000 trials, so a 0.05 margin
    /// is far from flaky).
    #[test]
    fn draw_scenario_uses_a_lower_zero_probability_for_the_first_food_type_than_others() {
        let mut rng = SmallRng::seed_from_u64(42);
        let trials = 4_000u32;
        let mut zero_first = 0u32;
        let mut zero_second = 0u32;
        for _ in 0..trials {
            let scenario = draw_scenario(&mut rng, 2);
            let food = &scenario.sensors.typed_local_food.food_here_by_type;
            if food[0] == 0.0 {
                zero_first += 1;
            }
            if food[1] == 0.0 {
                zero_second += 1;
            }
        }
        let frac_first = f64::from(zero_first) / f64::from(trials);
        let frac_second = f64::from(zero_second) / f64::from(trials);
        assert!(
            (frac_first - 0.5).abs() < 0.05,
            "food type 0's zero fraction should be near 0.5, got {frac_first}"
        );
        assert!(
            (frac_second - 0.6).abs() < 0.05,
            "food type 1's zero fraction should be near 0.6, got {frac_second}"
        );
    }
}
