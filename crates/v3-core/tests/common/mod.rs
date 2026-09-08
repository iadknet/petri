//! Shared test helpers for `v3-core` integration test binaries.
//!
//! Included via `#[path = "common/mod.rs"] mod common;` from each top-level
//! test file that needs it (`creature_workflow_e2e.rs`, `temporal_fixtures.rs`).
//! Cargo compiles each `tests/*.rs` file as a separate binary crate, so this
//! source is compiled once per binary rather than duplicated across files.

use slotmap::SlotMap;
use v3_core::config::SimulationConfig;
use v3_core::contracts::{CreatureId, Position};
use v3_core::creature::genome::CreatureGenome;
use v3_core::creature::identity::CreatureIdentityState;
use v3_core::creature::state::CreatureState;
use v3_core::kernel::WorldState;
use v3_core::runtime::trace::domain::{BackendTrace, GraphTrace, TickTrace};
use v3_core::runtime::trace::recording::ActiveTrace;
use v3_core::simulation::{run_tick, Simulation};

pub(crate) const FOUNDER_CHANNELS: [u8; 6] = [0, 0, 92, 92, 138, 138];
pub(crate) const FOUNDER_ACTIVE_CHANNEL: usize = 0;
pub(crate) const FOUNDER_POLARITY: [bool; 6] = [true; 6];

/// A tiny, zero-growth, zero-mutation world config for deterministic fixtures.
///
/// Production runtime settings (`RuntimeConfig::default()`) and shared-memory
/// decay are left untouched; callers that need a labeled deviation (for
/// example forcing a single graph relaxation pass, or a nonzero decay rate)
/// override the relevant field after calling this constructor.
pub(crate) fn test_config() -> SimulationConfig {
    let mut cfg = SimulationConfig::default();
    cfg.world.width = 12;
    cfg.world.height = 12;
    cfg.world.food.initial_coverage = 0.0;
    cfg.world.food.growth_rate = 0.0;
    // Both world-level Phase 0 energy charges are off, so a fixture that reads
    // a creature's energy difference across a tick reads only what the tick's
    // own behavior cost it.
    cfg.energy.lifecycle.energy_decay_per_tick = 0.0;
    cfg.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    cfg.mutation.mutation_probability = 0.0;
    cfg
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

/// Extract the graph trace for a mesh hop, panicking if that hop was a VM
/// dispatch instead.
pub(crate) fn graph_hop(tick: &TickTrace, hop_idx: usize) -> &GraphTrace {
    match &tick.hops[hop_idx].backend_trace {
        BackendTrace::Graph(g) => g,
        BackendTrace::Vm(_) => panic!("expected graph hop"),
    }
}
