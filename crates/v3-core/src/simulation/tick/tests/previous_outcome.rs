//! T19.F05 worked case V5 and the Phase 2.5 outcome store: every living
//! creature stores its tick's outcome bank, perception reads it frozen at
//! the next tick's start, and a newborn reads zeros.

use super::super::run_tick;
use super::support::*;
use crate::config::OrdinaryFoodTypeId;
use crate::contracts::{Direction, InputReference, NodeId};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, OutcomeChannel, VmBackendDef, VmInstruction,
};
use crate::sensors::static_inputs::assemble_static_inputs;

/// A one-node VM genome over `PreviousOutcome` running `program` with the
/// constants `[1.0, 0.0]`.
fn outcome_genome(program: Vec<VmInstruction>) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![InputReference::PreviousOutcome],
            backend_def: BackendDef::Vm(VmBackendDef {
                register_count: 2,
                constants: vec![1.0, 0.0],
                program,
            }),
            targets: vec![],
        }],
    }
}

/// `Move N = PreviousOutcome[ActionSuccess]`, or `Move N = 0` in the control.
fn v5_genome(read: bool) -> CreatureGenome {
    let load = if read {
        VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: OutcomeChannel::ActionSuccess as u16,
        }
    } else {
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 1,
        }
    };
    outcome_genome(vec![
        load,
        VmInstruction::AddVote {
            sink: VoteSink::Move(0).index() as u8,
            src: 0,
        },
        VmInstruction::Halt,
    ])
}

/// A genome voting one unit into `sink`.
fn voting_genome(sink: VoteSink) -> CreatureGenome {
    outcome_genome(vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::AddVote {
            sink: sink.index() as u8,
            src: 0,
        },
        VmInstruction::Halt,
    ])
}

#[test]
fn v5_previous_outcome_moves_on_the_tick_after_a_successful_noop() {
    for (read, moved_on_tick_two) in [(false, false), (true, true)] {
        let (mut sim, id) = make_sim_with_custom_genome(50.0, v5_genome(read));
        let start = sim.creatures[id].position;
        let north = sim
            .world
            .resolve_neighbor(start, Direction::N)
            .expect("a wrapped world has every neighbor");

        // Tick one reads the zeros of a creature with no full tick yet:
        // `NoOp`, which succeeds.
        run_tick(&mut sim, &mut None);
        assert_eq!(sim.creatures[id].position, start, "read={read}");
        assert_eq!(
            sim.creatures[id].previous_outcome[OutcomeChannel::ActionSuccess as usize],
            1.0
        );

        run_tick(&mut sim, &mut None);
        let expected = if moved_on_tick_two { north } else { start };
        assert_eq!(sim.creatures[id].position, expected, "read={read}");
    }
}

#[test]
fn an_eat_on_food_is_read_next_tick_as_its_scaled_outcome() {
    let (mut sim, id) = make_sim_with_custom_genome(50.0, voting_genome(VoteSink::Eat));
    let pos = sim.creatures[id].position;
    sim.world
        .set_food_type(pos, OrdinaryFoodTypeId::new(0), 1.0);

    run_tick(&mut sim, &mut None);

    let creature = &sim.creatures[id];
    let lifecycle = &sim.config.energy.lifecycle;
    let stored = creature.previous_outcome;
    assert!(
        stored[OutcomeChannel::EnergyDelta as usize] > 0.0,
        "{stored:?}"
    );
    assert_eq!(
        assemble_static_inputs(&sim.world, creature, lifecycle).previous_outcome,
        [
            stored[OutcomeChannel::EnergyDelta as usize] / lifecycle.max_energy,
            1.0,
            0.0,
            0.0
        ]
    );
}

#[test]
fn a_newborn_stores_zeros_and_its_parent_counts_the_birth() {
    // Vote Reproduce N with transfer fraction 1 (field 1,
    // `ReproduceTransferFraction`), capped at the default litter.
    let genome = outcome_genome(vec![
        VmInstruction::LoadConst {
            dst: 0,
            const_idx: 0,
        },
        VmInstruction::WriteActionParam {
            field_idx: 1,
            src: 0,
        },
        VmInstruction::AddVote {
            sink: VoteSink::Reproduce(0).index() as u8,
            src: 0,
        },
        VmInstruction::Halt,
    ]);
    let (mut sim, parent) = make_sim_with_custom_genome(150.0, genome);
    sim.config.energy.lifecycle.min_reproduce_age = 0;
    sim.config.energy.lifecycle.min_reproduce_energy = 0.0;

    run_tick(&mut sim, &mut None);

    assert_eq!(sim.creatures.len(), 2, "the parent reproduced");
    let parent_outcome = sim.creatures[parent].previous_outcome;
    assert_eq!(
        parent_outcome[OutcomeChannel::OffspringSuccess as usize],
        1.0
    );
    assert_eq!(parent_outcome[OutcomeChannel::ActionSuccess as usize], 1.0);
    let (_, child) = sim
        .creatures
        .iter()
        .find(|(id, _)| *id != parent)
        .expect("the newborn");
    assert_eq!(child.previous_outcome, [0.0; 4]);
    let lifecycle = &sim.config.energy.lifecycle;
    assert_eq!(
        assemble_static_inputs(&sim.world, child, lifecycle).previous_outcome,
        [0.0; 4]
    );
}
