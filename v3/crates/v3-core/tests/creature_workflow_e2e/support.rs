use slotmap::SlotMap;
use v3_core::config::SimulationConfig;
use v3_core::contracts::{CreatureId, NodeId, Position};
use v3_core::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use v3_core::creature::identity::CreatureIdentityState;
use v3_core::creature::state::CreatureState;
use v3_core::kernel::WorldState;
use v3_core::runtime::trace::{ActiveTrace, BackendTrace, TickTrace};
use v3_core::simulation::{run_tick, Simulation};

pub(crate) const FOUNDER_CHANNELS: [u8; 6] = [0, 0, 92, 92, 138, 138];
pub(crate) const FOUNDER_ACTIVE_CHANNEL: usize = 0;
pub(crate) const FOUNDER_POLARITY: [bool; 6] = [true; 6];

pub(crate) fn test_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 12;
    cfg.world.height = 12;
    cfg.world.food.initial_coverage = 0.0;
    cfg.world.food.growth_rate = 0.0;
    cfg.energy.lifecycle.energy_decay_per_tick = 0.0;
    cfg.mutation.mutation_probability = 0.0;
    cfg
}

pub(crate) fn vm_emit_noop_genome() -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 1,
                constants: vec![],
                program: vec![
                    VmInstruction::PushAction { action_type: 0 },
                    VmInstruction::ExecuteActionQueue,
                ],
            }),
            targets: vec![],
        }],
    }
}

pub(crate) fn insert_creature(
    creatures: &mut SlotMap<CreatureId, CreatureState>,
    world: &mut WorldState,
    genome: CreatureGenome,
    position: Position,
    energy: f32,
    generation: u64,
) -> CreatureId {
    let id = creatures.insert_with_key(|id| {
        CreatureState::new(
            id,
            genome,
            position,
            energy,
            generation,
            FOUNDER_CHANNELS,
            FOUNDER_ACTIVE_CHANNEL,
            FOUNDER_POLARITY,
            CreatureIdentityState::default(),
            [0.0; 16],
        )
    });
    world.place_creature(position, id);
    id
}

pub(crate) fn run_one_traced_tick(sim: &mut Simulation, target: CreatureId) -> TickTrace {
    let mut trace = Some(ActiveTrace::new(target, 1));
    run_tick(sim, &mut trace);
    let trace = trace.expect("trace should remain present");
    assert!(trace.is_complete(), "single tick trace should complete");
    assert_eq!(trace.ticks.len(), 1, "expected exactly one traced tick");
    trace.ticks.into_iter().next().expect("missing traced tick")
}

pub(crate) fn graph_hop(tick: &TickTrace, hop_idx: usize) -> &v3_core::runtime::trace::GraphTrace {
    match &tick.hops[hop_idx].backend_trace {
        BackendTrace::Graph(g) => g,
        BackendTrace::Vm(_) => panic!("expected graph hop"),
    }
}

pub(crate) fn vm_hop(tick: &TickTrace, hop_idx: usize) -> &v3_core::runtime::trace::VmTrace {
    match &tick.hops[hop_idx].backend_trace {
        BackendTrace::Vm(v) => v,
        BackendTrace::Graph(_) => panic!("expected vm hop"),
    }
}
