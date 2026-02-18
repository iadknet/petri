use crate::config::SimulationConfig;
use crate::contracts::inputs::CreatureInputs;
use crate::contracts::outputs::{CreatureOutputs, WorldAction};
use crate::creature::genome::CreatureGenome;
use crate::creature::state::Energy;
use crate::runtime::vm;

/// Execute a creature's VM brain for one tick.
///
/// Looks up the entry node in the genome, runs the VM execution engine,
/// and returns `CreatureOutputs` with the emitted world action (or `NoOp`
/// if the program halted without calling `EmitWorldAction`).
///
/// Energy is drained per-opcode during VM execution. The caller (tick
/// orchestrator) must pass `&mut creature.energy` to enable this.
pub fn execute_vm_creature(
    genome: &CreatureGenome,
    inputs: &CreatureInputs,
    memory: &mut [u8],
    energy: &mut Energy,
    config: &SimulationConfig,
) -> CreatureOutputs {
    let entry_node = genome
        .nodes
        .iter()
        .find(|n| n.node_id == genome.entry_node_id)
        .unwrap_or_else(|| {
            panic!(
                "VM: entry_node_id {:?} not found in genome nodes",
                genome.entry_node_id
            )
        });

    let world_action = vm::execute_vm_node(entry_node, inputs, memory, energy, &config.runtime.vm)
        .unwrap_or(WorldAction::NoOp);

    CreatureOutputs {
        world_action,
        ..CreatureOutputs::noop()
    }
}
