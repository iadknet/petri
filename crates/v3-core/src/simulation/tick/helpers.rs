use crate::contracts::CreatureId;
use crate::runtime::types::MeshOutput;
use crate::simulation::simulation::Simulation;

pub(super) fn remove_creature_from_sim(sim: &mut Simulation, id: CreatureId) {
    sim.remove_creature(id);
}

pub(super) fn remove_creature_if_dead(sim: &mut Simulation, id: CreatureId) -> bool {
    if sim
        .creatures
        .get(id)
        .is_some_and(|creature| creature.energy <= 0.0)
    {
        remove_creature_from_sim(sim, id);
        return true;
    }
    false
}

pub(crate) fn sort_by_priority_bid(decisions: &mut [(CreatureId, MeshOutput)]) {
    decisions.sort_by(|a, b| {
        b.1.priority_bid
            .partial_cmp(&a.1.priority_bid)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
