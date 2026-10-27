use std::collections::{HashMap, HashSet};
use std::time::Instant;

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;

use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{
    CreatureId, Direction, PerceptionRequirements, Position, WorldAction, SHARED_MEMORY_SLOTS,
};
use crate::creature::action_log::{ActionLogEntry, ActionResult, ActionType, NO_DIRECTION};
use crate::creature::state::CreatureState;
use crate::genome::OUTCOME_CHANNEL_COUNT;
use crate::kernel::WorldState;
use crate::runtime::mesh::execute_creature_mesh;
use crate::runtime::trace::domain::{
    PerceptionDebugSnapshot, StaticInputsSnapshot, TerminationReason, TickTrace,
};
use crate::runtime::types::MeshOutput;
use crate::sensors::perception::{PerceptionConfig, PerceptionSnapshot, SensorSnapshot};
use crate::sensors::reducers::assemble_perception;
use crate::sensors::static_inputs::{assemble_static_inputs, BodyViews};
use crate::sensors::typed_food::{assemble_typed_food_local_snapshot, TypedFoodLocalSnapshot};
use crate::sensors::visibility::{
    compute_visible_cells_into, get_visibility_table, VisibilityScratch,
};
use crate::simulation::actions::{
    apply_give_energy, apply_move, apply_noop, apply_reproduce, apply_steal_energy,
    apply_typed_eat, deliver_pending_birth, BarrierReaderState, BirthDelivery, GiveActionResult,
    MoveBlockedCause, PredationActionResult, ReproductionActionResult,
    ReproductionInvalidTargetCause,
};
use crate::simulation::outcomes::OutcomeAccumulator;
use crate::simulation::recording::{ActiveTrace, RecordedTick};
use crate::simulation::simulation::Simulation;
use crate::simulation::stats::SimStats;

#[path = "tick/helpers.rs"]
mod helpers;

pub(crate) use self::helpers::sort_by_priority_bid;
use self::helpers::{remove_creature_from_sim, remove_creature_if_dead};

#[cfg(feature = "telemetry-seams")]
#[path = "tick/window_seam.rs"]
mod window_seam;
#[cfg(feature = "telemetry-seams")]
use self::window_seam::{record_limit, FlowMarks};
#[cfg(feature = "telemetry-seams")]
use crate::runtime::traced_mesh::RecordedMesh;
#[cfg(feature = "telemetry-seams")]
use crate::simulation::recording::{TickOutcome, Truncation};

#[cfg(test)]
#[path = "tick/tests/mod.rs"]
mod tests;

fn classify_move_blocked_cause(
    world: &WorldState,
    from: Position,
    dir: Direction,
) -> MoveBlockedCause {
    let Some(target) = world.resolve_neighbor(from, dir) else {
        return MoveBlockedCause::OutOfBounds;
    };
    if world.is_barrier(target) {
        return MoveBlockedCause::Barrier;
    }
    if world.creature_at(target).is_some() {
        return MoveBlockedCause::Occupied;
    }
    MoveBlockedCause::OutOfBounds
}

fn classify_reproduction_invalid_target_cause(
    world: &WorldState,
    target: Option<Position>,
    successful_spawn_targets: &HashSet<Position>,
) -> ReproductionInvalidTargetCause {
    let Some(target) = target else {
        return ReproductionInvalidTargetCause::OutOfBounds;
    };
    if successful_spawn_targets.contains(&target) {
        return ReproductionInvalidTargetCause::Contention;
    }
    if world.is_barrier(target) {
        return ReproductionInvalidTargetCause::Barrier;
    }
    if world.creature_at(target).is_some() {
        return ReproductionInvalidTargetCause::Occupied;
    }
    ReproductionInvalidTargetCause::OutOfBounds
}

fn has_neighbor_barrier(world: &WorldState, position: Position) -> bool {
    Direction::ALL.iter().any(|dir| {
        world
            .resolve_neighbor(position, *dir)
            .is_some_and(|neighbor| world.is_barrier(neighbor))
    })
}

fn has_any_valid_adjacent_target(world: &WorldState, position: Position) -> bool {
    Direction::ALL.iter().any(|dir| {
        world
            .resolve_neighbor(position, *dir)
            .is_some_and(|target| !world.is_barrier(target) && world.creature_at(target).is_none())
    })
}

fn barrier_reader_state_for_creature(creature: &CreatureState) -> BarrierReaderState {
    if creature.has_barrier_reader() {
        BarrierReaderState::HasBarrierReader
    } else {
        BarrierReaderState::NoBarrierReader
    }
}

/// Snapshot `shared_memory` into `prev_shared_memory`, then decay `shared_memory`
/// in place by `decay_rate` (a no-op multiply guard at the production default of
/// `0.0`). This is the one rule for how shared memory crosses a tick boundary;
/// `run_phase_0` and the mutational-neighborhood indicator's multi-tick battery
/// (T11.F01) both call it rather than duplicating the snapshot-then-decay order.
pub fn advance_shared_memory(
    shared_memory: &mut [f32; SHARED_MEMORY_SLOTS],
    prev_shared_memory: &mut [f32; SHARED_MEMORY_SLOTS],
    decay_rate: f32,
) {
    *prev_shared_memory = *shared_memory;
    if decay_rate > 0.0 {
        let factor = 1.0 - decay_rate;
        for slot in shared_memory.iter_mut() {
            *slot *= factor;
        }
    }
}

/// The energy one living creature is charged in the Phase 0 energy-decay
/// sub-step (T26.F02): the world-level `decay`, the `f32` `base`, that decay
/// plus the per-tick maintenance cost of the structure it carries, junk
/// included, and the body's signed capacity `upkeep`. Not scaled by the
/// complexity or age action multipliers, exactly as `energy_decay_per_tick`
/// is not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Phase0Charge {
    pub(crate) decay: f32,
    pub(crate) base: f32,
    pub(crate) upkeep: f32,
}

/// One creature's settled Phase 0 charge: its energy after, the applied
/// debit's three shares, and the sink a death it causes is attributed to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Phase0Settlement {
    pub(crate) after: f32,
    pub(crate) decay: f64,
    pub(crate) carrying: f64,
    pub(crate) upkeep: f64,
    pub(crate) cause: crate::ledger::DeathCause,
}

impl Phase0Charge {
    /// `base = decay + carry_per_unit × genome_size` in `f32`, beside the
    /// signed `upkeep`.
    pub(crate) fn new(decay: f32, carry_per_unit: f32, genome_size: u32, upkeep: f32) -> Self {
        Self {
            decay,
            base: decay + carry_per_unit * genome_size as f32,
            upkeep,
        }
    }

    /// The one `f32` subtraction Phase 0 applies, `max(base + upkeep, 0)`:
    /// a small body's negative upkeep never credits energy. At `upkeep = 0`
    /// it is `base` bit for bit.
    pub(crate) fn applied(self) -> f32 {
        (self.base + self.upkeep).max(0.0)
    }

    /// Charge `before`. Decay and carrying are booked by today's
    /// decay-first partition of `before − base`; the signed upkeep takes the
    /// applied debit's remainder. Attribution uses the charge's rounded
    /// intermediates: `LifecycleDecay` when `before − decay ≤ 0`, else
    /// `GenomeCarrying` when `before − base ≤ 0`, else `CapacityUpkeep`.
    pub(crate) fn settle(self, before: f32) -> Phase0Settlement {
        use crate::ledger::{applied_debit, decay_partition, DeathCause};
        let after = before - self.applied();
        let (decay_share, carrying) = decay_partition(before, before - self.base, self.decay);
        let cause = if before - self.decay <= 0.0 {
            DeathCause::LifecycleDecay
        } else if before - self.base <= 0.0 {
            DeathCause::GenomeCarrying
        } else {
            DeathCause::CapacityUpkeep
        };
        Phase0Settlement {
            after,
            decay: decay_share,
            carrying,
            upkeep: applied_debit(before, after) - decay_share - carrying,
            cause,
        }
    }
}

/// Run Phase 0 of a tick: food growth, creature aging, energy decay, dead-creature removal.
///
/// Sub-step canonical order (v3-tick-orchestration-spec.md Section 3):
/// 1. Food growth
/// 2. Creature aging (+1 per creature)
/// 3. Energy decay (subtract `energy_decay_per_tick` plus the genome carrying
///    cost `genome_carry_cost_per_unit * genome_size()` and the signed
///    capacity upkeep, floored at zero, as one charge: [`Phase0Charge`])
/// 4. Death removal (remove creatures where energy <= 0 from slotmap + world occupancy)
///
/// `run_tick` runs the same two halves with a clock between them: step 1 is
/// the `food_growth` phase, steps 2 to 4 the `world_update` phase (T24.F03).
pub fn run_phase_0(sim: &mut Simulation) {
    run_phase_0_food_growth(sim);
    run_phase_0_creatures(sim);
}

/// Phase 0 step 1: food growth. Under `telemetry-seams` it returns the timed
/// grid passes for the tick-trace seam.
#[cfg(feature = "telemetry-seams")]
fn run_phase_0_food_growth(sim: &mut Simulation) -> crate::kernel::FoodPasses {
    let mut food_growth = sim.world.grow_food(sim.tick, sim.food_stream_key);
    let passes = food_growth.take_passes();
    sim.stats.record_food_growth_summary(food_growth);
    passes
}

/// Phase 0 step 1: food growth.
#[cfg(not(feature = "telemetry-seams"))]
fn run_phase_0_food_growth(sim: &mut Simulation) {
    let food_growth = sim.world.grow_food(sim.tick, sim.food_stream_key);
    sim.stats.record_food_growth_summary(food_growth);
}

/// Phase 0 steps 2 to 4: aging, energy decay, death removal.
fn run_phase_0_creatures(sim: &mut Simulation) {
    // Steps 2 & 3: Age, energy decay, and shared memory snapshot + decay
    let decay_rate = sim.config.shared_memory.decay_rate;
    let lifecycle = &sim.config.energy.lifecycle;
    let capacity = &sim.config.energy.capacity;
    for (_, creature) in sim.creatures.iter_mut() {
        creature.age += 1;
        let energy_before = creature.energy;
        let settled = Phase0Charge::new(
            lifecycle.energy_decay_per_tick,
            lifecycle.genome_carry_cost_per_unit,
            creature.genome_size(),
            creature.body.upkeep(capacity),
        )
        .settle(energy_before);
        creature.energy = settled.after;
        let flows = &mut sim.stats.energy_flows;
        flows.lifecycle_decay += settled.decay;
        flows.genome_carrying += settled.carrying;
        flows.capacity_upkeep += settled.upkeep;
        flows.genome_size_creature_ticks += u64::from(creature.genome_size());
        creature.observe_energy(energy_before, settled.cause);
        creature.lifetime_energy_sum += f64::from(creature.energy.max(0.0));
        creature.lifetime_energy_sample_count += 1;

        creature.begin_tick();
        advance_shared_memory(
            &mut creature.shared_memory,
            &mut creature.prev_shared_memory,
            decay_rate,
        );
    }

    // Step 4: Collect dead IDs first to avoid borrow conflict during removal.
    let dead_ids: Vec<_> = sim
        .creatures
        .iter()
        .filter(|(_, c)| c.energy <= 0.0)
        .map(|(id, _)| id)
        .collect();

    for id in dead_ids {
        remove_creature_from_sim(sim, id);
    }
}

/// Phase 1a: assemble sensor inputs sequentially over the post-Phase-0 world.
///
/// Conditional perception: typed local food and full visibility + area
/// reduction only for creatures whose cached perception requirements ask for
/// them. Others use the `zero()` fallbacks.
fn assemble_sensor_inputs(
    sim: &Simulation,
    queue: &[CreatureId],
) -> Vec<(CreatureId, SensorSnapshot)> {
    assemble_sensor_inputs_with(sim, queue, CreatureState::perception_requirements)
}

/// The body view of every living creature, as sensors read it this tick.
#[must_use]
pub fn body_views(sim: &Simulation) -> BodyViews {
    let capacity = &sim.config.energy.capacity;
    sim.creatures
        .iter()
        .map(|(id, creature)| (id, creature.sensed_body(capacity)))
        .collect()
}

/// The production sensor assemblers with typed local food and extended
/// perception assembled for every creature, whatever its genome reads: the
/// complete snapshot a creature's cell and neighbors offer, for observation
/// panels that run other genomes on it (T11.F26 recorded contexts).
/// Skips ids that are not alive; never advances or mutates the simulation.
#[must_use]
pub fn assemble_full_sensor_inputs(
    sim: &Simulation,
    queue: &[CreatureId],
) -> Vec<(CreatureId, SensorSnapshot)> {
    assemble_sensor_inputs_with(sim, queue, |_| PerceptionRequirements::ALL)
}

#[inline]
fn assemble_sensor_inputs_with(
    sim: &Simulation,
    queue: &[CreatureId],
    requirements: impl Fn(&CreatureState) -> PerceptionRequirements,
) -> Vec<(CreatureId, SensorSnapshot)> {
    let perception_config = PerceptionConfig::from_sim_config(&sim.config);
    let vis_table = get_visibility_table(perception_config.vision_radius);
    let mut visible_scratch = VisibilityScratch::default();
    let views = body_views(sim);
    queue
        .iter()
        .filter(|&&id| sim.creatures.contains_key(id))
        .map(|&id| {
            let creature = &sim.creatures[id];
            let needs = requirements(creature);
            let local = assemble_static_inputs(&sim.world, &views[id], &sim.config.energy);
            let typed_local_food = if needs.typed_local_food {
                assemble_typed_food_local_snapshot(&sim.world, creature.position)
            } else {
                TypedFoodLocalSnapshot::zero()
            };
            let perception = if needs.extended_perception {
                let visible = compute_visible_cells_into(
                    creature.position,
                    &sim.world,
                    vis_table,
                    &mut visible_scratch,
                );
                assemble_perception(id, visible, &sim.world, &views, &perception_config)
            } else {
                PerceptionSnapshot::zero()
            };
            let ss = SensorSnapshot {
                world_reads: Default::default(),
                local,
                typed_local_food,
                perception,
            };
            (id, ss)
        })
        .collect()
}

/// The complete action queue selected for one final-state observation.
///
/// The three queues are evaluated from the same frozen sensor snapshot. Each
/// evaluation owns fresh copies of the mutable cognition state, so this is
/// observational and never changes the simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct FinalActionObservation {
    pub creature_id: CreatureId,
    pub intact: Vec<WorldAction>,
    pub zeroed: Vec<WorldAction>,
    pub scrambled: Vec<WorldAction>,
}

/// Observe every living creature's selected action queue at the simulation's
/// final state without advancing or mutating the simulation.
///
/// Creature IDs are sorted before sensor assembly to make the returned order
/// stable. The scramble is a fixed left rotation of the 16 shared-memory
/// slots; it preserves the memory multiset and does not consume the simulation
/// RNG. The normal mesh executor itself has no RNG input.
pub fn observe_final_actions(sim: &Simulation) -> Vec<FinalActionObservation> {
    let mut creature_ids: Vec<_> = sim.creatures.keys().collect();
    creature_ids.sort();
    let inputs = assemble_sensor_inputs(sim, &creature_ids);

    inputs
        .into_iter()
        .map(|(creature_id, sensors)| {
            let creature = &sim.creatures[creature_id];
            let mut scrambled_memory = creature.shared_memory;
            scrambled_memory.rotate_left(1);
            FinalActionObservation {
                creature_id,
                intact: observe_action_queue(
                    creature,
                    &sensors,
                    creature.shared_memory,
                    &sim.config.runtime,
                ),
                zeroed: observe_action_queue(
                    creature,
                    &sensors,
                    [0.0; SHARED_MEMORY_SLOTS],
                    &sim.config.runtime,
                ),
                scrambled: observe_action_queue(
                    creature,
                    &sensors,
                    scrambled_memory,
                    &sim.config.runtime,
                ),
            }
        })
        .collect()
}

/// Named temporal substrates, each intervened on separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalMemorySubstrate {
    PreviousSlots,
    PersistedOutputs,
    OperatorState,
}

/// Full action comparisons at a synthetic next graph-cognition boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct TemporalActionObservation {
    pub substrate: TemporalMemorySubstrate,
    pub actions: FinalActionObservation,
}

#[derive(Clone, Copy)]
enum MemoryIntervention {
    Zero,
    Rotate,
}

/// Prepare the graph clock on cloned state, without advancing shared-memory
/// snapshot/decay, age, energy, world, or RNG. Then perturb only the named read
/// substrate; no further tick snapshot occurs before evaluating each clone.
/// Rotations are one position left within each individual slot vector.
pub fn observe_temporal_actions(sim: &Simulation) -> Vec<TemporalActionObservation> {
    let mut ids: Vec<_> = sim.creatures.keys().collect();
    ids.sort();
    let mut observations = Vec::with_capacity(ids.len() * 3);
    for (id, sensors) in assemble_sensor_inputs(sim, &ids) {
        let creature = &sim.creatures[id];
        let mut prepared = creature.graph_runtime().clone();
        prepared.begin_tick(creature.age);
        let evaluate = |graph: &mut crate::runtime::state::GraphRuntimeState,
                        previous: &[f32; SHARED_MEMORY_SLOTS]| {
            execute_creature_mesh(
                creature.genome(),
                &sensors,
                &mut { creature.energy },
                &mut { creature.shared_memory },
                previous,
                graph,
                &sim.config.runtime,
                creature.heading,
            )
            .actions
        };
        let intact = evaluate(&mut prepared.clone(), &creature.prev_shared_memory);
        for substrate in [
            TemporalMemorySubstrate::PreviousSlots,
            TemporalMemorySubstrate::PersistedOutputs,
            TemporalMemorySubstrate::OperatorState,
        ] {
            let [zeroed, scrambled] =
                [MemoryIntervention::Zero, MemoryIntervention::Rotate].map(|intervention| {
                    let mut graph = prepared.clone();
                    let mut previous = creature.prev_shared_memory;
                    let perturb = |slots: &mut [f32]| match intervention {
                        MemoryIntervention::Zero => slots.fill(0.0),
                        MemoryIntervention::Rotate => {
                            if !slots.is_empty() {
                                slots.rotate_left(1);
                            }
                        }
                    };
                    match substrate {
                        TemporalMemorySubstrate::PreviousSlots => perturb(&mut previous),
                        // Committed state is the live read base (T19.F02).
                        TemporalMemorySubstrate::PersistedOutputs => graph
                            .node_outputs
                            .iter_mut()
                            .for_each(|slots| perturb(slots)),
                        TemporalMemorySubstrate::OperatorState => {
                            graph.node_state.iter_mut().for_each(|slots| perturb(slots))
                        }
                    }
                    evaluate(&mut graph, &previous)
                });
            observations.push(TemporalActionObservation {
                substrate,
                actions: FinalActionObservation {
                    creature_id: id,
                    intact: intact.clone(),
                    zeroed,
                    scrambled,
                },
            });
        }
    }
    observations
}

fn observe_action_queue(
    creature: &CreatureState,
    sensors: &SensorSnapshot,
    mut shared_memory: [f32; SHARED_MEMORY_SLOTS],
    runtime_config: &crate::config::RuntimeConfig,
) -> Vec<WorldAction> {
    let mut energy = creature.energy;
    let mut graph_runtime = creature.graph_runtime().clone();
    graph_runtime.begin_tick(creature.age);
    execute_creature_mesh(
        creature.genome(),
        sensors,
        &mut energy,
        &mut shared_memory,
        &creature.prev_shared_memory,
        &mut graph_runtime,
        runtime_config,
        creature.heading,
    )
    .actions
}

/// Phase 1b: cognition — parallel for all creatures, sequential for the traced one.
///
/// Cognition only mutates each creature's private state (energy, memory,
/// `graph_runtime`); the world snapshot stays frozen for the whole phase.
fn run_cognition(
    sim: &mut Simulation,
    inputs: &[(CreatureId, SensorSnapshot)],
    trace: &mut Option<ActiveTrace>,
    trace_target: Option<CreatureId>,
) -> Vec<(CreatureId, MeshOutput)> {
    // Clone RuntimeConfig for cognition phase (small struct, ~7 scalars).
    let runtime_config = sim.config.runtime.clone();
    let tick_number = sim.tick;
    #[cfg(feature = "telemetry-seams")]
    let limit = trace
        .as_ref()
        .and_then(|active| record_limit(active, &sim.config));
    let mut creature_refs: HashMap<_, _> = sim.creatures.iter_mut().collect();

    // Extract traced creature (if any) before building parallel work vec.
    let traced_creature = trace_target.and_then(|tid| creature_refs.remove(&tid).map(|c| (tid, c)));

    let mut work: Vec<_> = inputs
        .iter()
        .filter_map(|(id, ss)| creature_refs.remove(id).map(|c| (*id, ss, c)))
        .collect();

    // Run all non-traced creatures in parallel.
    let mut parallel_decisions: Vec<_> = work
        .par_iter_mut()
        .map(|(id, ss, creature)| {
            let output = creature.think(ss, &runtime_config);
            creature.pending_death_cause = output.energy_observation.pending_cause;
            creature.heading = output.heading;
            creature.last_mesh_world_reads = ss.world_reads.set();
            (*id, output)
        })
        .collect();

    // Run traced creature sequentially with trace recording.
    if let Some((tid, creature)) = traced_creature {
        if let Some((_, ss)) = inputs.iter().find(|(id, _)| *id == tid) {
            let energy_before = creature.energy;
            let heading_before = creature.heading;
            let si_snapshot = StaticInputsSnapshot::from(&ss.local);

            #[cfg(feature = "telemetry-seams")]
            let (position, age) = (creature.position, creature.age);
            #[cfg(feature = "telemetry-seams")]
            let RecordedMesh {
                output,
                hops,
                passes,
                truncated,
            } = creature.think_budgeted(ss, &runtime_config, limit);
            #[cfg(not(feature = "telemetry-seams"))]
            let (output, hops, passes) = creature.think_traced(ss, &runtime_config);
            creature.pending_death_cause = output.energy_observation.pending_cause;
            creature.heading = output.heading;
            creature.last_mesh_world_reads = ss.world_reads.set();

            // Record tick trace.
            if let Some(ref mut active) = trace {
                let debug_perception = if active.include_perception_debug {
                    Some(PerceptionDebugSnapshot::from(&ss.perception))
                } else {
                    None
                };
                active.ticks.push(RecordedTick {
                    trace: TickTrace {
                        tick_number,
                        heading: heading_before,
                        energy_before,
                        energy_after: creature.energy,
                        static_inputs: si_snapshot,
                        debug_perception,
                        hops,
                        passes,
                        final_actions: output.actions.clone(),
                        termination_reason: output.termination_reason,
                        priority_bid: output.priority_bid,
                        commit_counts: output.commit_counts,
                    },
                    #[cfg(feature = "telemetry-seams")]
                    outcome: Some(TickOutcome {
                        position,
                        age,
                        typed_local_food: ss.typed_local_food.clone(),
                        typed_area_food: ss.perception.typed_area_food.clone(),
                        uses_typed_local_food: creature.perception_requirements().typed_local_food,
                        uses_extended_perception: creature
                            .perception_requirements()
                            .extended_perception,
                        applied: Vec::with_capacity(runtime_config.max_actions_per_turn),
                        damage_received: 0.0,
                        offspring_spawned: 0,
                        after: None,
                        died: None,
                        phases: None,
                    }),
                });
                active.ticks_remaining = active.ticks_remaining.saturating_sub(1);
                #[cfg(feature = "telemetry-seams")]
                if active.budget.is_some() {
                    if let Some(record) = active.ticks.last_mut() {
                        record.trace.hops.shrink_to_fit();
                        record.trace.passes.shrink_to_fit();
                        record.trace.final_actions.shrink_to_fit();
                    }
                    if let Some(reason) = truncated {
                        active.truncated = Some(Truncation {
                            tick: tick_number,
                            reason,
                        });
                        active.ticks_remaining = 0;
                    }
                }
            }

            // Insert traced creature's decision at its queue position.
            // Find where tid appears in the original queue order.
            let queue_pos = inputs.iter().position(|(id, _)| *id == tid);
            // Count how many non-traced inputs precede it to find the
            // insertion point in parallel_decisions.
            if let Some(pos) = queue_pos {
                let insert_idx = inputs[..pos].iter().filter(|(id, _)| *id != tid).count();
                parallel_decisions.insert(insert_idx, (tid, output));
            } else {
                parallel_decisions.push((tid, output));
            }
        }
    }

    parallel_decisions
}

/// Per-tick compute-cost and deterministic work counters accumulated during
/// Phase 2, committed to [`SimStats`] once the creature queue is fully processed.
struct TickComputeStats {
    total_sum: f32,
    total_min: f32,
    total_max: f32,
    vm_sum: f32,
    vm_count: u32,
    graph_sum: f32,
    graph_count: u32,
    creature_count: u32,
    priority_bid_sum: f32,
    priority_bid_count: u32,
    mesh_hops: u64,
    vm_steps: u64,
    graph_relax_iters: u64,
    plasticity_updates: u64,
    plasticity_changes: u64,
    shared_memory_writes_changed: u64,
    energy_exhausted_dispatches: u64,
    pass_cap_hits: u64,
    passes: u64,
    decided_passes: u64,
}

impl Default for TickComputeStats {
    fn default() -> Self {
        Self {
            total_sum: 0.0,
            total_min: f32::MAX,
            total_max: 0.0,
            vm_sum: 0.0,
            vm_count: 0,
            graph_sum: 0.0,
            graph_count: 0,
            creature_count: 0,
            priority_bid_sum: 0.0,
            priority_bid_count: 0,
            mesh_hops: 0,
            vm_steps: 0,
            graph_relax_iters: 0,
            plasticity_updates: 0,
            plasticity_changes: 0,
            shared_memory_writes_changed: 0,
            energy_exhausted_dispatches: 0,
            pass_cap_hits: 0,
            passes: 0,
            decided_passes: 0,
        }
    }
}

impl TickComputeStats {
    /// Accumulate one creature's mesh output in queue order.
    ///
    /// Integer work counters commute exactly, so summing in queue order is
    /// deterministic regardless of the parallel mesh phase's thread scheduling.
    fn record(&mut self, output: &MeshOutput) {
        let compute_cost = &output.cost_report;
        let total_cost =
            compute_cost.vm_cost + compute_cost.graph_cost + compute_cost.mesh_ramp_cost;
        self.total_sum += total_cost;
        if total_cost < self.total_min {
            self.total_min = total_cost;
        }
        if total_cost > self.total_max {
            self.total_max = total_cost;
        }
        if compute_cost.vm_cost > 0.0 {
            self.vm_sum += compute_cost.vm_cost;
            self.vm_count += 1;
        }
        if compute_cost.graph_cost > 0.0 {
            self.graph_sum += compute_cost.graph_cost;
            self.graph_count += 1;
        }
        self.creature_count += 1;

        let work = output.work_counters;
        self.mesh_hops += u64::from(work.mesh_hops);
        self.vm_steps += u64::from(work.vm_steps);
        self.graph_relax_iters += u64::from(work.graph_relax_iters);
        self.plasticity_updates += u64::from(work.plasticity_updates);
        self.plasticity_changes += u64::from(work.plasticity_changes);
        self.shared_memory_writes_changed += u64::from(work.shared_memory_writes_changed);
        self.pass_cap_hits += u64::from(work.pass_cap_hits);
        self.passes += u64::from(work.passes);
        self.decided_passes += u64::from(work.decided_passes);
        if output.termination_reason == TerminationReason::EnergyExhausted {
            self.energy_exhausted_dispatches += 1;
        }

        self.priority_bid_sum += output.priority_bid;
        if output.priority_bid > 0.0 {
            self.priority_bid_count += 1;
        }
    }

    /// Write per-tick compute stats and commit deterministic work counters.
    ///
    /// `plasticity_updates_total` also receives the reward-modulated
    /// contribution from Phase 2.5, which runs after `decisions` is consumed
    /// and cannot flow through `MeshOutput`.
    fn commit(&self, stats: &mut SimStats) {
        if self.creature_count > 0 {
            stats.last_tick_compute_total_mean = self.total_sum / self.creature_count as f32;
            stats.last_tick_compute_total_min = self.total_min;
            stats.last_tick_compute_total_max = self.total_max;
            stats.last_tick_compute_vm_mean = if self.vm_count > 0 {
                self.vm_sum / self.vm_count as f32
            } else {
                0.0
            };
            stats.last_tick_compute_graph_mean = if self.graph_count > 0 {
                self.graph_sum / self.graph_count as f32
            } else {
                0.0
            };
            stats.last_tick_priority_bid_mean = self.priority_bid_sum / self.creature_count as f32;
            stats.last_tick_priority_bidders_count = self.priority_bid_count;
        }

        stats.mesh_hops_total += self.mesh_hops;
        stats.vm_steps_total += self.vm_steps;
        stats.graph_relax_iters_total += self.graph_relax_iters;
        stats.plasticity_updates_total += self.plasticity_updates;
        stats.hebbian_updates_total += self.plasticity_updates;
        stats.plasticity_changes_total += self.plasticity_changes;
        stats.hebbian_changes_total += self.plasticity_changes;
        stats.shared_memory_writes_changed_total += self.shared_memory_writes_changed;
        stats.creature_ticks_total += u64::from(self.creature_count);
        stats.mesh_dispatches_energy_exhausted_total += self.energy_exhausted_dispatches;
        stats.pass_cap_hits_total += self.pass_cap_hits;
        stats.passes_total += self.passes;
        stats.decided_passes_total += self.decided_passes;
        stats.actions_applied_total += u64::from(
            stats.last_tick_move
                + stats.last_tick_eat
                + stats.last_tick_noop
                + stats.last_tick_reproduce
                + stats.last_tick_steal
                + stats.last_tick_give,
        );
    }
}

/// Per-action bookkeeping shared by every Phase 2 action executor.
#[derive(Clone, Copy)]
struct ActionContext {
    id: CreatureId,
    tick: u64,
    priority_bid: f32,
    /// Creature energy captured before this action was applied.
    energy_before: f32,
}

/// Append one entry to a creature's action log, reading energy after the action.
fn push_action_log(
    sim: &mut Simulation,
    ctx: &ActionContext,
    action_type: ActionType,
    result: ActionResult,
    direction: u8,
    amount: f32,
    food_type: Option<OrdinaryFoodTypeId>,
) -> ActionLogEntry {
    let energy_after = sim.creatures.get(ctx.id).map_or(0.0, |c| c.energy);
    let entry = ActionLogEntry {
        tick: ctx.tick,
        action_type,
        result,
        direction,
        energy_before: ctx.energy_before,
        energy_after,
        amount,
        food_type,
        priority_bid: ctx.priority_bid,
    };
    #[cfg(all(test, feature = "telemetry-seams"))]
    window_seam::capture_logged(ctx.id, entry);
    if let Some(log) = sim.action_logs.get_mut(ctx.id) {
        log.push(entry);
    }
    entry
}

/// Barrier-awareness telemetry captured before a move or reproduce attempt.
///
/// `position` is `None` when the creature is already gone, in which case the
/// remaining fields carry the neutral "no barrier awareness" values.
#[derive(Clone, Copy)]
struct BarrierContext {
    position: Option<Position>,
    has_barrier_neighbor: bool,
    reader_state: BarrierReaderState,
    has_alternative_target: bool,
}

impl BarrierContext {
    fn for_creature(sim: &Simulation, id: CreatureId) -> Self {
        let Some(creature) = sim.creatures.get(id) else {
            return Self {
                position: None,
                has_barrier_neighbor: false,
                reader_state: BarrierReaderState::NoBarrierReader,
                has_alternative_target: false,
            };
        };
        let position = creature.position;
        Self {
            position: Some(position),
            has_barrier_neighbor: has_neighbor_barrier(&sim.world, position),
            reader_state: barrier_reader_state_for_creature(creature),
            has_alternative_target: has_any_valid_adjacent_target(&sim.world, position),
        }
    }
}

/// NoOp cannot fail; its effort is the floor every action's effort is held to.
fn execute_noop(
    sim: &mut Simulation,
    ctx: &ActionContext,
    outcome_acc: &mut OutcomeAccumulator,
) -> ActionLogEntry {
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.record_action_attempt(ActionType::NoOp);
        apply_noop(creature, &sim.config, &mut sim.stats.energy_flows);
        sim.stats.last_tick_noop += 1;
        outcome_acc.record_action_result(ctx.id, true);
    }
    push_action_log(
        sim,
        ctx,
        ActionType::NoOp,
        ActionResult::Success,
        NO_DIRECTION,
        0.0,
        None,
    )
}

fn execute_eat(
    sim: &mut Simulation,
    ctx: &ActionContext,
    type_idx: OrdinaryFoodTypeId,
    leave: f32,
    outcome_acc: &mut OutcomeAccumulator,
) -> ActionLogEntry {
    let mut action_result = ActionResult::Success;
    let mut amount = 0.0;
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.record_action_attempt(ActionType::Eat);
        let food_before = sim.world.food_at_type(creature.position, type_idx);
        let eaten = apply_typed_eat(
            creature,
            &mut sim.world,
            &sim.config,
            type_idx,
            leave,
            &mut sim.stats.energy_flows,
        );
        let succeeded = eaten > 0.0;
        sim.stats.last_tick_eat += 1;
        outcome_acc.record_action_result(ctx.id, succeeded);
        if succeeded {
            amount = eaten;
            creature.record_applied_eat(type_idx);
            *sim.stats
                .eat_actions_applied_total_by_type
                .entry(type_idx)
                .or_insert(0) += 1;
            if eaten < food_before {
                *sim.stats
                    .eat_actions_partial_total_by_type
                    .entry(type_idx)
                    .or_insert(0) += 1;
            }
        } else {
            action_result = ActionResult::NoFood;
            *sim.stats
                .eat_actions_failed_total_by_type
                .entry(type_idx)
                .or_insert(0) += 1;
        }
    }
    push_action_log(
        sim,
        ctx,
        ActionType::Eat,
        action_result,
        NO_DIRECTION,
        amount,
        Some(type_idx),
    )
}

fn execute_move(
    sim: &mut Simulation,
    ctx: &ActionContext,
    dir: Direction,
    outcome_acc: &mut OutcomeAccumulator,
) -> ActionLogEntry {
    let mut action_result = ActionResult::Success;
    let mut blocked_cause = None;
    let barrier = BarrierContext::for_creature(sim, ctx.id);
    if barrier.has_barrier_neighbor {
        *sim.stats
            .move_attempts_with_barrier_neighbor_total_by_reader_state
            .entry(barrier.reader_state)
            .or_insert(0) += 1;
    }
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.record_action_attempt(ActionType::Move);
        let from = creature.position;
        let succeeded = apply_move(
            ctx.id,
            creature,
            &mut sim.world,
            dir,
            &sim.config,
            &mut sim.stats.energy_flows,
        );
        sim.stats.last_tick_move += 1;
        sim.stats.move_actions_attempted_total += 1;
        outcome_acc.record_action_result(ctx.id, succeeded);
        if !succeeded {
            creature.lifetime_blocked_move_count += 1;
            blocked_cause = Some(classify_move_blocked_cause(&sim.world, from, dir));
            action_result = ActionResult::Blocked;
        }
    }
    if let Some(cause) = blocked_cause {
        *sim.stats
            .move_actions_blocked_total_by_cause
            .entry(cause)
            .or_insert(0) += 1;
        if barrier.has_alternative_target {
            *sim.stats
                .move_actions_blocked_avoidable_total_by_reader_state
                .entry(barrier.reader_state)
                .or_insert(0) += 1;
        }
        if barrier.has_barrier_neighbor && matches!(cause, MoveBlockedCause::Barrier) {
            *sim.stats
                .move_blocked_barrier_with_barrier_neighbor_total_by_reader_state
                .entry(barrier.reader_state)
                .or_insert(0) += 1;
        }
    }
    push_action_log(
        sim,
        ctx,
        ActionType::Move,
        action_result,
        dir.to_index() as u8,
        0.0,
        None,
    )
}

fn execute_reproduce(
    sim: &mut Simulation,
    ctx: &ActionContext,
    direction: Direction,
    energy_transfer_fraction: f32,
    outcome_acc: &mut OutcomeAccumulator,
    reproduce_rng: &mut SmallRng,
    successful_spawn_targets: &mut HashSet<Position>,
) -> ActionLogEntry {
    let barrier = BarrierContext::for_creature(sim, ctx.id);
    let reproduction_target = barrier
        .position
        .and_then(|pos| sim.world.resolve_neighbor(pos, direction));
    if barrier.has_barrier_neighbor {
        *sim.stats
            .reproduction_attempts_with_barrier_neighbor_total_by_reader_state
            .entry(barrier.reader_state)
            .or_insert(0) += 1;
    }
    let result = apply_reproduce(
        ctx.id,
        sim,
        direction,
        energy_transfer_fraction,
        reproduce_rng,
    );
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.record_action_attempt(ActionType::Reproduce);
    }
    let succeeded = result == ReproductionActionResult::Conceived;
    outcome_acc.record_action_result(ctx.id, succeeded);
    // A conception with no pending birth left was delivered at once (span 0,
    // T03.F12) on the validated target; a carried litter is recorded at its
    // delivery instead.
    let born_now = succeeded
        && sim
            .creatures
            .get(ctx.id)
            .is_some_and(|parent| parent.pending_birth.is_none());
    if born_now {
        if let Some(target) = reproduction_target {
            successful_spawn_targets.insert(target);
        }
        outcome_acc.record_offspring(ctx.id);
    }
    if result == ReproductionActionResult::RejectedInvalidTarget {
        if let Some(creature) = sim.creatures.get_mut(ctx.id) {
            creature.lifetime_invalid_reproduce_count += 1;
        }
        let cause = classify_reproduction_invalid_target_cause(
            &sim.world,
            reproduction_target,
            successful_spawn_targets,
        );
        *sim.stats
            .reproduction_actions_rejected_invalid_target_total_by_cause
            .entry(cause)
            .or_insert(0) += 1;
        if barrier.has_alternative_target {
            *sim.stats
                .reproduction_actions_rejected_invalid_target_avoidable_total_by_reader_state
                .entry(barrier.reader_state)
                .or_insert(0) += 1;
        }
        if barrier.has_barrier_neighbor && matches!(cause, ReproductionInvalidTargetCause::Barrier)
        {
            *sim.stats
                .reproduction_invalid_target_barrier_with_barrier_neighbor_total_by_reader_state
                .entry(barrier.reader_state)
                .or_insert(0) += 1;
        }
    }
    let action_result = match result {
        ReproductionActionResult::Conceived => ActionResult::Success,
        ReproductionActionResult::RejectedInvalidTarget => ActionResult::InvalidTarget,
        ReproductionActionResult::RejectedAgeConstraints => ActionResult::AgeConstraints,
        ReproductionActionResult::RejectedEnergyConstraints => ActionResult::EnergyConstraints,
        ReproductionActionResult::RejectedPopulationCap => ActionResult::PopulationCap,
    };
    push_action_log(
        sim,
        ctx,
        ActionType::Reproduce,
        action_result,
        direction.to_index() as u8,
        energy_transfer_fraction,
        None,
    )
}

fn execute_steal_energy(
    sim: &mut Simulation,
    ctx: &ActionContext,
    direction: Direction,
    effort: f32,
    outcome_acc: &mut OutcomeAccumulator,
) -> ActionLogEntry {
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.record_action_attempt(ActionType::StealEnergy);
    }
    let outcome = apply_steal_energy(ctx.id, sim, direction, effort);
    let result = outcome.result;
    outcome_acc.record_action_result(ctx.id, result.transferred());
    // A surviving victim takes the damage; a killed one was removed.
    if let (PredationActionResult::Transferred, Some(victim_id)) = (result, outcome.counterpart) {
        outcome_acc.record_damage(victim_id, outcome.amount);
    }
    let action_result = match result {
        PredationActionResult::Transferred => ActionResult::Success,
        PredationActionResult::TransferredAndKilled => ActionResult::TransferredAndKilled,
        PredationActionResult::RejectedNoVictim | PredationActionResult::RejectedNothingToTake => {
            ActionResult::NoVictim
        }
    };
    push_action_log(
        sim,
        ctx,
        ActionType::StealEnergy,
        action_result,
        direction.to_index() as u8,
        outcome.amount,
        None,
    )
}

fn execute_give_energy(
    sim: &mut Simulation,
    ctx: &ActionContext,
    direction: Direction,
    effort: f32,
    outcome_acc: &mut OutcomeAccumulator,
) -> ActionLogEntry {
    if let Some(creature) = sim.creatures.get_mut(ctx.id) {
        creature.record_action_attempt(ActionType::GiveEnergy);
    }
    let outcome = apply_give_energy(ctx.id, sim, direction, effort);
    outcome_acc.record_action_result(ctx.id, outcome.result == GiveActionResult::Transferred);
    let action_result = match outcome.result {
        GiveActionResult::Transferred => ActionResult::Success,
        GiveActionResult::RejectedNoRecipient => ActionResult::NoRecipient,
        GiveActionResult::RejectedNothingToGive => ActionResult::EnergyConstraints,
    };
    push_action_log(
        sim,
        ctx,
        ActionType::GiveEnergy,
        action_result,
        direction.to_index() as u8,
        outcome.amount,
        None,
    )
}

/// Phase 2: sequential action execution.
///
/// Applies decisions in queue order, accumulating compute cost stats.
fn run_phase_2(
    sim: &mut Simulation,
    decisions: Vec<(CreatureId, MeshOutput)>,
    outcome_acc: &mut OutcomeAccumulator,
    reproduce_rng: &mut SmallRng,
) -> TickComputeStats {
    let mut compute = TickComputeStats::default();
    let mut successful_spawn_targets: HashSet<Position> = HashSet::new();

    for (id, output) in decisions {
        compute.record(&output);
        sim.stats
            .energy_flows
            .record_cognition(output.energy_observation);

        // Skip all actions for creatures killed by earlier predation this tick.
        if !sim.creatures.contains_key(id) {
            continue;
        }

        // Gestation hold (T03.F12): a parent carrying a litter commits
        // nothing; on the span's last tick its child is born here first.
        if let Some(pending) = sim.creatures[id].pending_birth {
            // Phase 1 charges can exhaust a creature without removing it.
            if remove_creature_if_dead(sim, id) {
                continue;
            }
            if sim.tick >= pending.deliver_tick {
                if let BirthDelivery::Delivered(cell) =
                    deliver_pending_birth(id, sim, reproduce_rng)
                {
                    successful_spawn_targets.insert(cell);
                    outcome_acc.record_offspring(id);
                }
            }
            sim.stats.gestation_creature_ticks_total += 1;
            continue;
        }

        // Apply each queued action sequentially, recording to action log.
        let current_tick = sim.tick;
        let priority_bid = output.priority_bid;
        for action in &output.actions {
            // Capture energy before action (creature may have been killed).
            let energy_before = match sim.creatures.get(id) {
                Some(c) => c.energy,
                None => break,
            };
            let ctx = ActionContext {
                id,
                tick: current_tick,
                priority_bid,
                energy_before,
            };

            #[cfg(feature = "telemetry-seams")]
            let marks = (sim.stats.observed_creature == Some(id))
                .then(|| FlowMarks::of(&sim.stats.energy_flows));
            let charged_before = (
                sim.stats.energy_flows.action_charges,
                sim.stats.energy_flows.failed_action_effort,
            );
            #[cfg_attr(not(feature = "telemetry-seams"), allow(unused_variables))]
            let entry = match *action {
                WorldAction::NoOp => execute_noop(sim, &ctx, outcome_acc),
                WorldAction::Eat { type_idx, leave } => {
                    execute_eat(sim, &ctx, type_idx, leave, outcome_acc)
                }
                WorldAction::Move(dir) => execute_move(sim, &ctx, dir, outcome_acc),
                WorldAction::Reproduce {
                    direction,
                    energy_transfer_fraction,
                } => execute_reproduce(
                    sim,
                    &ctx,
                    direction,
                    energy_transfer_fraction,
                    outcome_acc,
                    reproduce_rng,
                    &mut successful_spawn_targets,
                ),
                WorldAction::StealEnergy { direction, effort } => {
                    execute_steal_energy(sim, &ctx, direction, effort, outcome_acc)
                }
                WorldAction::GiveEnergy { direction, effort } => {
                    execute_give_energy(sim, &ctx, direction, effort, outcome_acc)
                }
            };
            // Floor energy at 0.0 — creatures cannot spend more than they have.
            // The action's own overdraft nets out of its charge; only a
            // deficit it inherited is a floor credit.
            if let Some(creature) = sim.creatures.get_mut(id) {
                let before = creature.energy;
                creature.energy = creature.energy.max(0.0);
                let overshoot = crate::ledger::applied_debit(creature.energy, before);
                let flows = &mut sim.stats.energy_flows;
                flows.zero_floor_credit += flows.net_action_overdraft(charged_before, overshoot);
            }

            #[cfg(feature = "telemetry-seams")]
            if let Some(marks) = marks {
                let applied = marks.applied(entry, &sim.stats.energy_flows);
                sim.stats.observed_actions.push(applied);
            }

            if remove_creature_if_dead(sim, id) {
                break;
            }
            // A conception ends the queue: the rest is discarded with no
            // charge and no log entry (T03.F12). Held creatures never reach
            // this loop, so a pending birth here was conceived just now.
            if sim.creatures[id].pending_birth.is_some() {
                break;
            }
        }
    }

    compute
}

/// Phase 2.5: the outcome store and the reward-modulated learning pass.
///
/// Every living creature stores this tick's outcome signal bank as its
/// `previous_outcome` (T19.F05): energy now minus the post-Phase-0 snapshot,
/// before the zero floor; a creature born this tick has no
/// record and stores zeros. For each creature with a reward-modulated edge,
/// the same bank then drives the three-factor update of every initialized
/// reward-modulated edge from its eligibility trace (T25.F06).
fn run_reward_learning(sim: &mut Simulation, outcome_acc: &OutcomeAccumulator) {
    use crate::ledger::applied_debit;

    for (id, creature) in sim.creatures.iter_mut() {
        // Newborns spawned this tick have no outcome record.
        let signals = outcome_acc.compute_signal_bank(id, creature.energy);
        creature.previous_outcome = signals
            .as_ref()
            .map_or([0.0; OUTCOME_CHANNEL_COUNT], |bank| bank.signals);

        let Some(signals) =
            signals.filter(|_| creature.graph_runtime().learned().has_reward_modulated())
        else {
            continue;
        };
        let counts = creature.apply_reward(&signals);
        sim.stats.plasticity_updates_total += u64::from(counts.updates);
        sim.stats.reward_modulated_updates_total += u64::from(counts.updates);
        sim.stats.plasticity_changes_total += u64::from(counts.changes);
        sim.stats.reward_modulated_changes_total += u64::from(counts.changes);
        // Floor energy at 0.0 — same invariant as Phase 2 action costs. Learning
        // is free (T25.F02), so the floor is its own step: it reaches exactly
        // the creatures with an outcome bank and a reward-modulated edge.
        let before = creature.energy;
        creature.energy = creature.energy.max(0.0);
        sim.stats.energy_flows.zero_floor_credit += applied_debit(creature.energy, before);
    }
}

/// Run one full simulation tick per v3-tick-orchestration-spec.md.
///
/// Two-phase model:
/// 1. Phase 0: food growth (timed as `food_growth`), then aging, decay and
///    death removal (timed as `world_update`)
/// 2. Build turn queue: sort all creature IDs then shuffle
/// 3. Phase 1 — Batch cognition: all creatures see the frozen post-Phase-0 world snapshot
/// 4. Phase 2 — Sequential action execution: apply decisions in queue order
/// 5. Increment sim.tick
///
/// The optional `trace` parameter enables execution tracing for a single creature.
/// When `Some`, the target creature is extracted from the parallel batch and run
/// sequentially with `execute_creature_mesh_traced`, recording detailed trace data.
/// When `None`, behavior is identical to the untraced path.
pub fn run_tick(sim: &mut Simulation, trace: &mut Option<ActiveTrace>) {
    // Reset per-tick counters at the start of each tick.
    sim.stats.reset_tick_counters();
    #[cfg(feature = "telemetry-seams")]
    {
        sim.stats.observed_creature = window_seam::observed(trace);
    }

    #[cfg(feature = "telemetry-seams")]
    let population_start = sim.creatures.len();
    #[cfg(feature = "telemetry-seams")]
    let mut phases = [crate::simulation::stats::PhaseTiming::default(); 6];
    let phase_started = Instant::now();
    #[cfg(feature = "telemetry-seams")]
    let tick_started = phase_started;
    // Records one phase for the tick-trace seam: its offset from the tick's
    // start and the same `Duration` `phase_wall_clock` gains.
    #[cfg(feature = "telemetry-seams")]
    let mut record = |index: usize, started: Instant, elapsed| {
        phases[index] = crate::simulation::stats::PhaseTiming {
            offset: started.duration_since(tick_started),
            elapsed,
        };
    };
    #[cfg(not(feature = "telemetry-seams"))]
    let record = |_: usize, _: Instant, _: std::time::Duration| {};
    // ── Phase 0: food growth, then aging, energy decay and death removal ──
    #[cfg(feature = "telemetry-seams")]
    let food_passes = run_phase_0_food_growth(sim);
    #[cfg(not(feature = "telemetry-seams"))]
    run_phase_0_food_growth(sim);
    let elapsed = phase_started.elapsed();
    sim.stats.phase_wall_clock.food_growth += elapsed;
    record(0, phase_started, elapsed);

    let phase_started = Instant::now();
    run_phase_0_creatures(sim);
    let elapsed = phase_started.elapsed();
    sim.stats.phase_wall_clock.world_update += elapsed;
    record(1, phase_started, elapsed);

    // Snapshot surviving creature energies for Phase 2.5 reward learning.
    let mut outcome_acc = OutcomeAccumulator::default();
    for (id, creature) in sim.creatures.iter() {
        outcome_acc.snapshot_energy(id, creature.energy);
    }

    // Build turn queue: stable sort for reproducibility, then shuffle.
    let mut queue: Vec<_> = sim.creatures.keys().collect();
    queue.sort();
    queue.shuffle(&mut sim.rng);

    // Check if traced creature died during Phase 0.
    if let Some(ref mut active) = trace {
        if !active.is_complete() && !sim.creatures.contains_key(active.creature_id) {
            active.ticks_remaining = 0;
            #[cfg(feature = "telemetry-seams")]
            {
                active.removed = sim.stats.removal_of(active.creature_id);
            }
        }
    }
    #[cfg(feature = "telemetry-seams")]
    window_seam::reserve_tick(trace, &sim.config, sim.tick);

    // Derive a separate RNG for reproduction to avoid double-borrowing sim.rng.
    let mut reproduce_rng = SmallRng::seed_from_u64(sim.rng.next_u64());

    // Determine if we need to trace a specific creature this tick.
    let trace_target: Option<CreatureId> = trace
        .as_ref()
        .filter(|t| !t.is_complete())
        .map(|t| t.creature_id);

    // ── Phase 1: Batch cognition (parallel, with optional trace extraction) ──
    let phase_started = Instant::now();
    let inputs = assemble_sensor_inputs(sim, &queue);
    let elapsed = phase_started.elapsed();
    sim.stats.phase_wall_clock.sensor_assembly += elapsed;
    record(2, phase_started, elapsed);

    let phase_started = Instant::now();
    let mut decisions = run_cognition(sim, &inputs, trace, trace_target);
    let elapsed = phase_started.elapsed();
    sim.stats.phase_wall_clock.cognition += elapsed;
    record(3, phase_started, elapsed);

    // Sort decisions by priority bid descending. Stable sort preserves the
    // pre-existing random shuffle order among creatures with equal bids.
    sort_by_priority_bid(&mut decisions);
    #[cfg(all(test, feature = "telemetry-seams"))]
    window_seam::capture_decided(&decisions);

    // ── Phase 2: Sequential action execution ────────────────────────────────
    let phase_started = Instant::now();
    let compute = run_phase_2(sim, decisions, &mut outcome_acc, &mut reproduce_rng);
    compute.commit(&mut sim.stats);
    let elapsed = phase_started.elapsed();
    sim.stats.phase_wall_clock.actions += elapsed;
    record(4, phase_started, elapsed);

    // ── Phase 2.5: Reward-modulated learning pass ────────────────────────
    let phase_started = Instant::now();
    run_reward_learning(sim, &outcome_acc);
    let elapsed = phase_started.elapsed();
    sim.stats.phase_wall_clock.reward_learning += elapsed;
    record(5, phase_started, elapsed);

    sim.tick += 1;
    #[cfg(feature = "telemetry-seams")]
    {
        sim.stats.last_tick_phases = Some(crate::simulation::stats::TickPhaseTimings {
            tick: sim.tick,
            population_start,
            threads: rayon::current_num_threads(),
            started: tick_started,
            elapsed: tick_started.elapsed(),
            phases,
        });
        // Same-size elements: `collect` reuses `grow`'s buffer in place.
        sim.stats.last_tick_food_passes = food_passes
            .into_iter()
            .map(|pass| crate::simulation::stats::FoodPassSpan {
                pass: pass.pass,
                food_type: pass.food_type,
                timing: crate::simulation::stats::PhaseTiming {
                    offset: pass.started.duration_since(tick_started),
                    elapsed: pass.elapsed,
                },
            })
            .collect();
        window_seam::finish_tick(sim, trace, &outcome_acc, sim.tick - 1);
    }
}

#[cfg(all(test, feature = "telemetry-seams"))]
mod phase_seam_tests;

#[cfg(test)]
mod final_action_observation_tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::contracts::{Direction, NodeId, OrdinaryFoodTypeId, WorldAction};
    use crate::genome::cgp::{
        CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
    };
    use crate::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
    use crate::simulation::seed_simulation;
    use crate::simulation::seeding::seed_simulation_with_founder_genomes;
    use proptest::prelude::*;
    use rand::RngCore;

    fn memory_direction_genome() -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Vm(VmBackendDef {
                    register_count: 3,
                    constants: vec![0.5, 1.0],
                    // Reproduce NE when slot 0 exceeds 0.5, else N, with
                    // slot 0 as the transfer fraction (`ReproduceTransferFraction`).
                    program: vec![
                        VmInstruction::LoadSlotImm {
                            dst: 0,
                            slot_idx: 0,
                        },
                        VmInstruction::LoadConst {
                            dst: 1,
                            const_idx: 0,
                        },
                        VmInstruction::CmpGt { dst: 2, a: 0, b: 1 },
                        VmInstruction::WriteActionParam {
                            field_idx:
                                crate::genome::vote::ActionParamField::ReproduceTransferFraction
                                    .index() as u8,
                            src: 0,
                        },
                        VmInstruction::AddVote { sink: 10, src: 2 },
                        VmInstruction::LoadConst {
                            dst: 1,
                            const_idx: 1,
                        },
                        VmInstruction::Sub { dst: 1, a: 1, b: 2 },
                        VmInstruction::AddVote { sink: 9, src: 1 },
                        VmInstruction::Halt,
                    ],
                }),
                targets: vec![],
            }],
        }
    }

    fn runtime_state_action_genome() -> CreatureGenome {
        let runtime_output = GraphEdge::new(GraphSource::ComputeNode(0), 1.0);
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Graph(
                    CgpGraphBackendDef::new_with_fixed_outputs()
                        .with_compute_nodes(vec![ComputeNode::new(
                            ComputeNodeKind::DecayIntegrator(0.0),
                            vec![],
                        )])
                        .with_output_sinks(vec![crate::genome::cgp::OutputSink::new(
                            crate::genome::cgp::OutputSinkKind::ActionVote(
                                crate::genome::vote::VoteSink::Eat,
                            ),
                            vec![runtime_output],
                        )]),
                ),
                targets: vec![],
            }],
        }
    }

    fn expected_memory_action(value: f32) -> Vec<WorldAction> {
        vec![WorldAction::Reproduce {
            direction: if value > 0.5 {
                Direction::NE
            } else {
                Direction::N
            },
            energy_transfer_fraction: value,
        }]
    }

    #[test]
    fn a_tick_copies_the_world_keys_the_mesh_read_onto_the_creature() {
        use crate::contracts::{OrdinaryFoodTypeId, WorldInputKey};
        let mut config = SimulationConfig::default();
        config.world.width = 16;
        config.world.height = 16;
        config.population.initial_creatures = 1;
        let mut sim = seed_simulation(config, 11);
        let id = sim.creatures.keys().next().expect("one founder");
        assert!(sim.creatures[id].last_mesh_world_reads.is_empty());

        run_tick(&mut sim, &mut None);

        let reads = &sim.creatures[id].last_mesh_world_reads;
        assert!(
            reads.contains(WorldInputKey::food_here(OrdinaryFoodTypeId::new(0))),
            "the founder's entry node reads food on its cell"
        );
        assert!(
            !reads.contains(WorldInputKey::NeighborOccupiedRing),
            "the founder declares the occupied ring but never reads it"
        );
    }

    #[test]
    fn final_action_observation_uses_full_actions_and_leaves_simulation_unchanged() {
        let mut config = SimulationConfig::default();
        config.world.width = 16;
        config.world.height = 16;
        config.population.initial_creatures = 1;
        let mut sim = seed_simulation_with_founder_genomes(config, 11, |_, founder| {
            *founder = memory_direction_genome();
        });
        let id = sim.creatures.keys().next().expect("one founder");
        let creature = &mut sim.creatures[id];
        creature.shared_memory[0] = 1.0;
        creature.shared_memory[1] = 0.25;
        creature.prev_shared_memory[0] = 0.75;
        let runtime = creature.runtime_mut();
        runtime.node_state = vec![vec![0.5]];
        runtime.scratch_prev = vec![1.0];
        runtime.scratch_curr = vec![2.0];
        runtime.scratch_backup = vec![3.0];
        runtime.scratch_w_inputs = vec![4.0];

        let memory_before = creature.shared_memory;
        let previous_memory_before = creature.prev_shared_memory;
        let energy_before = creature.energy;
        let graph_state_before = creature.graph_runtime().clone();
        let mut rng_before = sim.rng.clone();

        let observations = observe_final_actions(&sim);
        let _ = observe_temporal_actions(&sim);

        assert_eq!(observations.len(), 1);
        let observation = &observations[0];
        assert_eq!(observation.creature_id, id);
        assert_eq!(
            observation.intact,
            vec![WorldAction::Reproduce {
                direction: Direction::NE,
                energy_transfer_fraction: 1.0,
            }]
        );
        assert_ne!(observation.intact, observation.zeroed);
        assert_ne!(observation.intact, observation.scrambled);
        assert_eq!(sim.creatures[id].shared_memory, memory_before);
        assert_eq!(sim.creatures[id].prev_shared_memory, previous_memory_before);
        assert_eq!(sim.creatures[id].energy, energy_before);
        assert_eq!(
            sim.creatures[id].graph_runtime().node_state,
            graph_state_before.node_state
        );
        assert_eq!(
            sim.creatures[id].graph_runtime().learned(),
            graph_state_before.learned()
        );
        assert_eq!(
            sim.creatures[id].graph_runtime().scratch_prev,
            graph_state_before.scratch_prev
        );
        assert_eq!(
            sim.creatures[id].graph_runtime().scratch_curr,
            graph_state_before.scratch_curr
        );
        assert_eq!(
            sim.creatures[id].graph_runtime().scratch_backup,
            graph_state_before.scratch_backup
        );
        assert_eq!(
            sim.creatures[id].graph_runtime().scratch_w_inputs,
            graph_state_before.scratch_w_inputs
        );
        assert_eq!(sim.rng.next_u64(), rng_before.next_u64());
    }

    #[test]
    fn final_action_observation_preserves_learned_runtime_for_memory_insensitive_controller() {
        let mut config = SimulationConfig::default();
        config.world.width = 16;
        config.world.height = 16;
        config.population.initial_creatures = 1;
        let mut sim = seed_simulation_with_founder_genomes(config, 11, |_, founder| {
            *founder = runtime_state_action_genome();
        });
        let id = sim.creatures.keys().next().expect("one founder");
        let creature = &mut sim.creatures[id];
        creature.shared_memory[0] = 1.0;
        creature.shared_memory[1] = 0.25;
        creature.runtime_mut().node_state = vec![vec![1.0]];

        let observations = observe_final_actions(&sim);

        let expected = vec![WorldAction::eat(OrdinaryFoodTypeId::new(0))];
        let observation = observations.first().expect("one observation");
        assert_eq!(observation.intact, expected);
        assert_eq!(observation.zeroed, expected);
        assert_eq!(observation.scrambled, expected);
        assert_eq!(
            sim.creatures[id].graph_runtime().node_state,
            vec![vec![1.0]]
        );
    }

    #[test]
    fn temporal_probe_detects_each_substrate_and_preserves_live_state() {
        for substrate in [
            TemporalMemorySubstrate::PreviousSlots,
            TemporalMemorySubstrate::PersistedOutputs,
            TemporalMemorySubstrate::OperatorState,
        ] {
            let mut config = SimulationConfig::default();
            config.population.initial_creatures = 1;
            let source = match substrate {
                TemporalMemorySubstrate::PreviousSlots => GraphSource::SharedMemory {
                    slot: 0,
                    previous: true,
                },
                _ => GraphSource::ComputeNode(0),
            };
            let edge = GraphEdge::new(source, 1.0);
            let kind = if substrate == TemporalMemorySubstrate::OperatorState {
                ComputeNodeKind::DecayIntegrator(0.0)
            } else {
                ComputeNodeKind::Add
            };
            let mut genome = runtime_state_action_genome();
            let BackendDef::Graph(def) = &mut genome.nodes[0].backend_def else {
                unreachable!()
            };
            *def = def.clone().with_compute_nodes(vec![
                ComputeNode::new(kind, vec![edge]),
                ComputeNode::new(ComputeNodeKind::Constant(0.0), vec![]),
            ]);
            crate::genome::cgp::test_support::set_surface_plasticity(
                def,
                crate::genome::cgp::EdgeSurface::ComputeInput(0),
                Some(crate::genome::PlasticityConfig {
                    rule: crate::genome::HebbianRule::Classic,
                    learning_rate: 0.5,
                    weight_clamp: 2.0,
                    lamarckian: false,
                    modulation: Some(crate::genome::RewardModulationConfig {
                        reward_source: crate::genome::OutcomeChannel::EnergyDelta,
                        trace_decay: 0.5,
                    }),
                    retention: 1.0,
                }),
            );
            let mut sim = seed_simulation_with_founder_genomes(config, 11, |_, founder| {
                *founder = genome.clone();
            });
            let id = sim.creatures.keys().next().unwrap();
            let creature = &mut sim.creatures[id];
            creature.prev_shared_memory[0] = 1.0;
            // A non-Lamarckian learner's value and trace are post-birth state.
            let runtime = creature.runtime_mut();
            runtime.learned.set_slot(0, 0, 1.0, 0.5);
            runtime.node_state = vec![vec![1.0, 0.0]];
            runtime.node_outputs = vec![vec![1.0, 0.0]];
            let graph_before = creature.graph_runtime().clone();
            let memory_before = creature.shared_memory;
            let previous_before = creature.prev_shared_memory;
            let energy_before = creature.energy;
            let mut rng = sim.rng.clone();
            let _ = observe_final_actions(&sim);
            let observations = observe_temporal_actions(&sim);
            assert_eq!(observations.len(), 3);
            for observation in observations {
                if observation.substrate == substrate {
                    assert_ne!(observation.actions.intact, observation.actions.zeroed);
                    assert_ne!(observation.actions.intact, observation.actions.scrambled);
                } else {
                    assert_eq!(observation.actions.intact, observation.actions.zeroed);
                    assert_eq!(observation.actions.intact, observation.actions.scrambled);
                }
            }
            let creature = &sim.creatures[id];
            assert_eq!(creature.graph_runtime().learned(), graph_before.learned());
            assert_eq!(creature.graph_runtime().node_state, graph_before.node_state);
            assert_eq!(
                creature.graph_runtime().node_outputs,
                graph_before.node_outputs
            );
            assert_eq!(creature.shared_memory, memory_before);
            assert_eq!(creature.prev_shared_memory, previous_before);
            assert_eq!(creature.energy, energy_before);
            assert_eq!(sim.rng.next_u64(), rng.next_u64());
        }
    }

    proptest! {
        #[test]
        fn observation_scramble_reads_slot_one_while_intact_reads_slot_zero(
            slot_zero in 0.0f32..1.0,
            slot_one in 0.0f32..1.0,
        ) {
            let mut config = SimulationConfig::default();
            config.world.width = 16;
            config.world.height = 16;
            config.population.initial_creatures = 1;
            let mut sim = seed_simulation_with_founder_genomes(config, 11, |_, founder| {
                *founder = memory_direction_genome();
            });
            let id = sim.creatures.keys().next().expect("one founder");
            let creature = &mut sim.creatures[id];
            creature.shared_memory[0] = slot_zero;
            creature.shared_memory[1] = slot_one;

            let observation = observe_final_actions(&sim).pop().expect("one observation");

            prop_assert_eq!(observation.intact, expected_memory_action(slot_zero));
            prop_assert_eq!(observation.zeroed, expected_memory_action(0.0));
            prop_assert_eq!(observation.scrambled, expected_memory_action(slot_one));
            prop_assert_eq!(sim.creatures[id].shared_memory[0], slot_zero);
            prop_assert_eq!(sim.creatures[id].shared_memory[1], slot_one);
        }
    }
}
