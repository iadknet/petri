use crate::config::SimulationConfig;
use crate::contracts::outputs::WorldAction;
use crate::kernel::types::CreatureId;
use crate::tick::actions;
use crate::SimulationState;
use rand::seq::SliceRandom;
use rand::Rng;

/// Per-tick observability counters (GP-04).
pub struct TickStats {
    pub births: u32,
    pub deaths: u32,
    pub actions_attempted: u32,
    pub actions_succeeded: u32,
}

/// Full Phase 0-3 tick.
pub fn tick(
    state: &mut SimulationState,
    config: &SimulationConfig,
    rng: &mut impl Rng,
) -> TickStats {
    // Phase 0: World mechanics + energy decay
    state.world.grow_food(&config.world.food, rng);

    for (_, creature) in state.creatures.iter_mut() {
        creature.age += 1;
        creature
            .energy
            .drain_saturating(config.energy.lifecycle.energy_decay_per_tick);
    }

    // Phase 1: Cognition — gather inputs, run VM brain, collect action queue.
    // Uses iter_mut() because the VM executor drains energy per-opcode.
    // SimulationState has public fields so state.creatures and state.world
    // can be borrowed disjointly inside the loop.
    let mut action_queue: Vec<(CreatureId, WorldAction)> = Vec::new();
    for (id, creature) in state.creatures.iter_mut() {
        let inputs = crate::sensors::gather_inputs(creature, &state.world);
        let outputs = crate::runtime::executor::execute_vm_creature(
            &creature.genome,
            &inputs,
            &mut creature.memory,
            &mut creature.energy,
            config,
        );
        action_queue.push((id, outputs.world_action));
    }

    // Phase 2: Shuffle queue and execute actions
    let mut shuffled_queue = action_queue;
    shuffled_queue.shuffle(rng);

    let mut actions_attempted: u32 = 0;
    let mut actions_succeeded: u32 = 0;
    let mut spawns = Vec::new();

    for (creature_id, action) in &shuffled_queue {
        actions_attempted += 1;
        if let Some(creature) = state.creatures.get_mut(*creature_id) {
            let result = actions::execute_action(
                action,
                *creature_id,
                creature,
                &mut state.world,
                config,
                rng,
            );
            if result.succeeded() {
                actions_succeeded += 1;
            }
            if let Some(offspring) = result.offspring {
                spawns.push(offspring);
            }
        }
    }

    let mut births: u32 = 0;
    for offspring in spawns {
        if state.world.is_barrier(offspring.position) || state.world.is_occupied(offspring.position)
        {
            continue;
        }
        state.spawn_creature(offspring);
        births += 1;
    }

    // Phase 3: Cleanup — remove dead creatures and update spatial index
    let dead_ids: Vec<CreatureId> = state
        .creatures
        .iter()
        .filter(|(_, c)| !c.energy.is_alive())
        .map(|(id, _)| id)
        .collect();
    for id in &dead_ids {
        if let Some(creature) = state.creatures.remove(*id) {
            state.world.remove_creature(creature.position);
        }
    }
    let deaths = dead_ids.len() as u32;

    state.tick_number += 1;

    TickStats {
        births,
        deaths,
        actions_attempted,
        actions_succeeded,
    }
}
