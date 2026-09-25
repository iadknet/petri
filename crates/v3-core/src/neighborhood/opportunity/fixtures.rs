//! Adequacy fixtures and competence checks of the authored controllers:
//! does competent authored use behave as intended, and does it leave every
//! other founder decision alone? Each is a fixed set of single-tick contexts
//! run from fresh cognition state, exactly as `neighborhood-v1` snapshots.

use crate::config::{EnergyLifecycleConfig, OrdinaryFoodTypeId, RuntimeConfig};
use crate::contracts::{Direction, WorldAction};
use crate::creature::genome::CreatureGenome;
use crate::runtime::mesh::UntracedMeshExecution;
use crate::runtime::MeshOutput;
use crate::sensors::perception::{food_idx, PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::{age_fraction, StaticInputs};
use crate::sensors::typed_food::TypedFoodLocalSnapshot;

use super::super::battery::{execute_scenario_tick, Scenario};
use super::super::steering::SteeringBattery;
use super::super::Battery;
use super::controllers::{controller, Family};

/// The configured vision radius every goal world uses; the area summary's
/// offsets are normalized by it and its distances by `sqrt(2) * radius`.
const VISION_RADIUS: f32 = 5.0;
/// Food types in every fixture: grass (0) and fruit (1).
const FOOD_TYPES: usize = 2;

/// Food on the creature's own cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Local {
    None,
    Grass,
    Fruit,
    GrassAndFruit,
}

impl Local {
    pub const ALL: [Self; 4] = [Self::None, Self::Grass, Self::Fruit, Self::GrassAndFruit];

    const fn grass(self) -> bool {
        matches!(self, Self::Grass | Self::GrassAndFruit)
    }

    const fn fruit(self) -> bool {
        matches!(self, Self::Fruit | Self::GrassAndFruit)
    }
}

/// One fixture context's description.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Context {
    pub local: Local,
    /// Primary (grass) food on the east ring cell.
    pub ring_food: bool,
    pub eligible: bool,
    /// A cardinal barrier, as a direction index.
    pub barrier: Option<u8>,
    /// Primary food two cells away in this cardinal direction index.
    pub far_food: Option<u8>,
}

impl Context {
    /// The single-tick scenario, with an area summary consistent with the
    /// food placed: the nearest of the here cell, the east ring cell and the
    /// far cell.
    pub(super) fn scenario(self) -> Scenario {
        let lifecycle = EnergyLifecycleConfig::default();
        let grass = if self.local.grass() { 1.0 } else { 0.0 };
        let fruit = if self.local.fruit() { 1.0 } else { 0.0 };
        let mut ring = [0.0f32; 8];
        if self.ring_food {
            ring[Direction::E.to_index()] = 1.0;
        }
        let mut barriers = [0.0f32; 8];
        if let Some(direction) = self.barrier {
            barriers[usize::from(direction)] = 1.0;
        }
        let mut food_cells: Vec<(i32, i32)> = Vec::new();
        if self.local.grass() {
            food_cells.push((0, 0));
        }
        if self.ring_food {
            food_cells.push(Direction::E.delta());
        }
        if let Some(direction) = self.far_food {
            let (dx, dy) = Direction::ALL[usize::from(direction)].delta();
            food_cells.push((2 * dx, 2 * dy));
        }
        let mut perception = PerceptionSnapshot::zeroed(FOOD_TYPES);
        if let Some(&(dx, dy)) = food_cells.iter().min_by_key(|(dx, dy)| dx * dx + dy * dy) {
            let summary = &mut perception.typed_area_food[0];
            summary[food_idx::TOTAL_RATIO] = food_cells.len() as f32 / 121.0;
            summary[food_idx::NEAREST_DX] = dx as f32 / VISION_RADIUS;
            summary[food_idx::NEAREST_DY] = dy as f32 / VISION_RADIUS;
            summary[food_idx::NEAREST_DIST] =
                ((dx * dx + dy * dy) as f32).sqrt() / (VISION_RADIUS * 2f32.sqrt());
            summary[food_idx::MAX_VALUE] = 1.0;
            perception.area_food = *summary;
        }
        // Eligible: past the age gate with energy above the founder's
        // threshold (0.16 of the maximum); otherwise the same age, low energy.
        let energy = if self.eligible { 80.0 } else { 20.0 };
        Scenario {
            sensors: SensorSnapshot {
                local: StaticInputs {
                    food_here: grass,
                    neighbor_food: ring,
                    neighbor_barrier: barriers,
                    neighbor_occupied: [0.0; 8],
                    age_ticks: age_fraction(50, lifecycle.age_reference_ticks),
                    max_energy: lifecycle.max_energy,
                    previous_outcome: [0.0; 4],
                },
                typed_local_food: TypedFoodLocalSnapshot {
                    food_here_by_type: vec![grass, fruit],
                    neighbor_food_by_type: vec![ring, [0.0; 8]],
                },
                perception,
            },
            energy,
        }
    }

    /// Whether any of `family`'s authored channels reads nonzero.
    fn signal(self, family: Family) -> bool {
        match family {
            Family::Ring => self.barrier.is_some_and(|d| d % 2 == 0),
            Family::Vector => {
                let scenario = self.scenario();
                let summary = scenario.sensors.perception.typed_area_food[0];
                family
                    .authored_sub_indices()
                    .iter()
                    .any(|&sub| summary[usize::from(sub)] != 0.0)
            }
            Family::Scalar => self.local.fruit(),
        }
    }
}

fn run(genome: &CreatureGenome, scenario: &Scenario, runtime: &RuntimeConfig) -> MeshOutput {
    execute_scenario_tick(genome, scenario, runtime, UntracedMeshExecution)
}

/// Every competence context of `family`: local food × ring food ×
/// eligibility, crossed with the family's own signal variants (each cardinal
/// barrier for the ring, far food in each cardinal direction for the vector).
#[must_use]
pub fn competence_contexts(family: Family) -> Vec<Context> {
    let variants: Vec<(Option<u8>, Option<u8>)> = match family {
        Family::Ring => [0u8, 2, 4, 6].map(|d| (Some(d), None)).to_vec(),
        Family::Vector => [0u8, 2, 4, 6].map(|d| (None, Some(d))).to_vec(),
        Family::Scalar => vec![(None, None)],
    };
    let mut contexts = Vec::new();
    for local in Local::ALL {
        for ring_food in [false, true] {
            for eligible in [false, true] {
                for &(barrier, far_food) in &variants {
                    contexts.push(Context {
                        local,
                        ring_food,
                        eligible,
                        barrier,
                        far_food,
                    });
                }
            }
        }
    }
    contexts
}

fn moves_into_barrier(queue: &[WorldAction], barrier: Option<u8>) -> bool {
    queue
        .iter()
        .any(|action| matches!(action, WorldAction::Move(d) if Some(d.to_index() as u8) == barrier))
}

fn without_moves(queue: &[WorldAction]) -> Vec<WorldAction> {
    queue
        .iter()
        .filter(|action| !matches!(action, WorldAction::Move(_)))
        .copied()
        .collect()
}

fn reproduces(queue: &[WorldAction]) -> Vec<WorldAction> {
    queue
        .iter()
        .filter(|action| matches!(action, WorldAction::Reproduce { .. }))
        .copied()
        .collect()
}

fn eats(queue: &[WorldAction]) -> Vec<OrdinaryFoodTypeId> {
    queue
        .iter()
        .filter_map(|action| match action {
            WorldAction::Eat { type_idx } => Some(*type_idx),
            _ => None,
        })
        .collect()
}

/// Whether `authored`'s queue differs from `founder`'s only by `family`'s
/// intended change in `context`, keeping every founder `Reproduce` and every
/// founder `Eat` (the scalar may retype it to fruit).
fn intended(
    family: Family,
    context: Context,
    founder: &[WorldAction],
    authored: &[WorldAction],
) -> bool {
    if founder == authored {
        return true;
    }
    match family {
        Family::Ring => {
            without_moves(founder) == without_moves(authored)
                && moves_into_barrier(founder, context.barrier)
                && !moves_into_barrier(authored, context.barrier)
        }
        Family::Vector => {
            without_moves(founder) == without_moves(authored)
                && !context.ring_food
                && !context.local.grass()
        }
        Family::Scalar => {
            let fruit = OrdinaryFoodTypeId::new(1);
            let (before, after) = (eats(founder), eats(authored));
            context.local.fruit()
                && reproduces(founder) == reproduces(authored)
                && after.iter().all(|&type_idx| type_idx == fruit)
                && (after.len() == before.len()
                    || (before.is_empty() && !context.local.grass() && after.len() == 1))
        }
    }
}

/// One controller pair's competence and adequacy reading.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Competence {
    /// `neighborhood-v1` executions whose authored channels read zero, and
    /// how many of them `A_k` or `Z_k` acts differently on from the founder.
    pub battery_zero_signal: u32,
    pub battery_violations: u32,
    /// Competence contexts, those with the family's signal present, and
    /// violations: a zero-signal context where `A_k` differs from the
    /// founder, a signal context whose difference is not the intended
    /// change, or any context where `Z_k` differs.
    pub contexts: u32,
    pub signal_contexts: u32,
    pub signal_changed: u32,
    pub violations: u32,
    /// The family's adequacy fixture: met by `A_k`, and the founder's result.
    pub adequacy: Adequacy,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Adequacy {
    pub rule: String,
    pub trials: u32,
    pub authored_met: u32,
    pub founder_met: u32,
    pub passed: bool,
}

fn battery_scenarios(battery: &Battery) -> Vec<&Scenario> {
    battery
        .snapshots()
        .iter()
        .chain(battery.sequences().iter().flatten())
        .collect()
}

/// Read `family`'s competence on `founder`.
#[must_use]
pub fn competence(
    founder: &CreatureGenome,
    family: Family,
    battery: &Battery,
    runtime: &RuntimeConfig,
    decay_rate: f32,
) -> Competence {
    let (authored, _) = controller(founder, family, false);
    let (inert, _) = controller(founder, family, true);
    let signatures =
        [founder, &authored, &inert].map(|genome| battery.signature(genome, runtime, decay_rate));
    let reference = family.reference();
    let mut battery_zero_signal = 0;
    let mut battery_violations = 0;
    let executions = signatures
        .each_ref()
        .map(|signature| signature.executions().collect::<Vec<_>>());
    for (index, scenario) in battery_scenarios(battery).into_iter().enumerate() {
        let zero = family.authored_sub_indices().iter().all(|&sub| {
            crate::runtime::inputs::resolve_input(
                &reference,
                sub,
                &crate::runtime::inputs::ResolveCtx {
                    sensors: &scenario.sensors,
                    upstream_slots: &[0.0; crate::runtime::OUTPUT_SLOT_COUNT],
                    energy: scenario.energy,
                    energy_consumed: 0.0,
                    action_queue: &crate::contracts::ActionQueue::new(1),
                    votes: &[0.0; crate::creature::genome::vote::VOTE_SINK_COUNT],
                    previous_pass_votes: &[0.0; crate::creature::genome::vote::VOTE_SINK_COUNT],
                    commit_counts: &[0; crate::creature::genome::vote::VOTE_KIND_COUNT],
                    mesh_hops: 0,
                },
            ) == 0.0
        });
        let founder_queue = executions[0][index];
        let inert_differs = executions[2][index] != founder_queue;
        if zero {
            battery_zero_signal += 1;
        }
        battery_violations +=
            u32::from(inert_differs || (zero && executions[1][index] != founder_queue));
    }

    let mut reading = Competence {
        battery_zero_signal,
        battery_violations,
        contexts: 0,
        signal_contexts: 0,
        signal_changed: 0,
        violations: 0,
        adequacy: adequacy(founder, &authored, family, runtime),
    };
    for context in competence_contexts(family) {
        let scenario = context.scenario();
        let [f, a, z] =
            [founder, &authored, &inert].map(|genome| run(genome, &scenario, runtime).actions);
        let signal = context.signal(family);
        reading.contexts += 1;
        reading.signal_contexts += u32::from(signal);
        reading.signal_changed += u32::from(signal && a != f);
        let ok = z == f
            && if signal {
                intended(family, context, &f, &a)
            } else {
                a == f
            };
        reading.violations += u32::from(!ok);
    }
    reading
}

fn leads_with(queue: &[WorldAction], action: WorldAction) -> bool {
    queue.first() == Some(&action)
}

/// The family's adequacy fixture.
fn adequacy(
    founder: &CreatureGenome,
    authored: &CreatureGenome,
    family: Family,
    runtime: &RuntimeConfig,
) -> Adequacy {
    match family {
        Family::Ring => {
            let steering = SteeringBattery::generate(FOOD_TYPES);
            let executed = std::collections::BTreeSet::new();
            let a = steering.read(authored, runtime, &executed);
            let f = steering.read(founder, runtime, &executed);
            Adequacy {
                rule: "steering-v1 (b): avoids in every founder-leading scenario; the founder \
                       avoids none"
                    .to_string(),
                trials: f.avoidance_trials as u32,
                authored_met: a.avoided as u32,
                founder_met: f.avoided as u32,
                passed: f.avoidance_trials > 0
                    && a.avoidance_trials == f.avoidance_trials
                    && a.avoided == a.avoidance_trials
                    && f.avoided == 0,
            }
        }
        Family::Vector => {
            let far = |direction: u8, local: Local| Context {
                local,
                ring_food: false,
                eligible: false,
                barrier: None,
                far_food: Some(direction),
            };
            let mut authored_met = 0;
            let mut founder_met = 0;
            let mut invariant = true;
            for direction in [0u8, 2, 4, 6] {
                let toward = WorldAction::Move(Direction::ALL[usize::from(direction)]);
                let empty = far(direction, Local::None).scenario();
                let a = run(authored, &empty, runtime).actions;
                authored_met += u32::from(leads_with(&a, toward));
                founder_met +=
                    u32::from(leads_with(&run(founder, &empty, runtime).actions, toward));
                let fruit =
                    run(authored, &far(direction, Local::Fruit).scenario(), runtime).actions;
                invariant &= fruit == a;
                for context in [
                    Context {
                        ring_food: true,
                        ..far(direction, Local::None)
                    },
                    far(direction, Local::Grass),
                ] {
                    let scenario = context.scenario();
                    invariant &= run(authored, &scenario, runtime).actions
                        == run(founder, &scenario, runtime).actions;
                }
            }
            Adequacy {
                rule: "empty ring, nearest food in each cardinal direction: leads toward it in \
                       all four, the founder not; ring food or primary food here: the founder's \
                       action; an empty and a fruit-only cell: the same action"
                    .to_string(),
                trials: 4,
                authored_met,
                founder_met,
                passed: authored_met == 4 && founder_met < 4 && invariant,
            }
        }
        Family::Scalar => {
            let cell = |local: Local| Context {
                local,
                ring_food: false,
                eligible: false,
                barrier: None,
                far_food: None,
            };
            let eat_fruit = WorldAction::eat(OrdinaryFoodTypeId::new(1));
            let authored_on = |local| run(authored, &cell(local).scenario(), runtime).actions;
            let founder_on = |local| run(founder, &cell(local).scenario(), runtime).actions;
            let authored_met = [Local::Fruit, Local::GrassAndFruit]
                .into_iter()
                .filter(|&local| leads_with(&authored_on(local), eat_fruit))
                .count() as u32;
            let founder_met = [Local::Fruit, Local::GrassAndFruit]
                .into_iter()
                .filter(|&local| leads_with(&founder_on(local), eat_fruit))
                .count() as u32;
            Adequacy {
                rule: "fruit only and fruit with grass: leads with Eat(1); grass only: the \
                       founder's action"
                    .to_string(),
                trials: 2,
                authored_met,
                founder_met,
                passed: authored_met == 2
                    && founder_met == 0
                    && authored_on(Local::Grass) == founder_on(Local::Grass),
            }
        }
    }
}
