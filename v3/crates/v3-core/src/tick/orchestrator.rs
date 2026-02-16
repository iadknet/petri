use crate::kernel::types::CreatureId;
use crate::SimulationState;
use rand::Rng;

/// Per-tick observability counters (GP-04). Stub for Stage 1.
pub struct TickStats {
    pub births: u32,
    pub deaths: u32,
}

/// Stage 1 tick: Only world mechanics and age increment (no cognition yet).
pub fn tick(state: &mut SimulationState, _rng: &mut impl Rng) -> TickStats {
    // Phase 0: World mechanics (stub for now - no food growth yet)

    // Increment creature age
    for (_, creature) in state.creatures.iter_mut() {
        creature.age += 1;
    }

    // Phase 1: Cognition (stub - not implemented in Stage 1)
    // Phase 2: Action execution (stub - not implemented in Stage 1)

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

    TickStats { births: 0, deaths }
}
