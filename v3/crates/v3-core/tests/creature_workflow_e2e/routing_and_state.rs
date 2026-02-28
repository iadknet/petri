use slotmap::SlotMap;
use v3_core::contracts::{CreatureId, NodeId, Position, WorldAction};
use v3_core::creature::genome::{
    BackendDef, CreatureGenome, GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind,
    NodeGenome, VmBackendDef, VmInstruction,
};
use v3_core::creature::state::CreatureState;
use v3_core::kernel::WorldState;
use v3_core::simulation::Simulation;

use crate::support::{insert_creature, run_one_traced_tick, test_config};

#[test]
fn routing_wraps_negative_index_to_reachable_downstream_node_e2e() {
    let cfg = test_config();
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let pos = Position::new(2, 2);
    world.set_food(pos, 1.0);

    let id_entry = NodeId::new(0);
    let id_noop = NodeId::new(1);
    let id_eat = NodeId::new(2);

    let entry = NodeGenome {
        node_id: id_entry,
        input_refs: vec![],
        backend_def: BackendDef::Graph(GraphBackendDef {
            internal_nodes: vec![
                GraphInternalNode {
                    kind: GraphNodeKind::Constant(-1.0),
                    inputs: vec![],
                    hebbian: None,
                },
                GraphInternalNode {
                    kind: GraphNodeKind::RouterOutput,
                    inputs: vec![GraphInput {
                        source_idx: 0,
                        weight: 1.0,
                    }],
                    hebbian: None,
                },
            ],
        }),
        targets: vec![id_noop, id_eat],
    };
    let noop = NodeGenome {
        node_id: id_noop,
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
    };
    let eat = NodeGenome {
        node_id: id_eat,
        input_refs: vec![],
        backend_def: BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: vec![],
            program: vec![
                VmInstruction::PushAction { action_type: 1 },
                VmInstruction::ExecuteActionQueue,
            ],
        }),
        targets: vec![],
    };

    let genome = CreatureGenome {
        entry_node_id: id_entry,
        nodes: vec![entry, noop, eat],
    };
    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let target = insert_creature(&mut creatures, &mut world, genome, pos, 80.0, 0);
    let mut sim = Simulation::new(world, creatures, 0, cfg, 19);

    let tick = run_one_traced_tick(&mut sim, target);

    assert_eq!(tick.hops.len(), 2);
    assert_eq!(tick.final_actions[0], WorldAction::Eat);
    assert!((tick.hops[0].route_target_idx - (-1.0)).abs() < 1e-6);
    assert_eq!(
        tick.hops[1].node_id, id_eat,
        "route=-1 with 2 targets should rem_euclid to index 1"
    );
}

#[test]
fn graph_state_persists_across_ticks_e2e() {
    let mut cfg = test_config();
    cfg.runtime.max_graph_relax_iters = 1;
    cfg.runtime.graph_convergence_stable_passes = 1;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    let pos = Position::new(7, 7);

    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(GraphBackendDef {
                internal_nodes: vec![
                    GraphInternalNode {
                        kind: GraphNodeKind::Constant(1.0),
                        inputs: vec![],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::DecayIntegrator(0.5),
                        inputs: vec![GraphInput {
                            source_idx: 0,
                            weight: 1.0,
                        }],
                        hebbian: None,
                    },
                    GraphInternalNode {
                        kind: GraphNodeKind::CustomOutput(0),
                        inputs: vec![GraphInput {
                            source_idx: 1,
                            weight: 1.0,
                        }],
                        hebbian: None,
                    },
                ],
            }),
            targets: vec![],
        }],
    };

    let mut creatures: SlotMap<CreatureId, CreatureState> = SlotMap::with_key();
    let target = insert_creature(&mut creatures, &mut world, genome, pos, 60.0, 0);
    let mut sim = Simulation::new(world, creatures, 0, cfg, 23);

    let tick1 = run_one_traced_tick(&mut sim, target);
    let tick2 = run_one_traced_tick(&mut sim, target);

    let out1 = tick1.hops[0].output_slots[0];
    let out2 = tick2.hops[0].output_slots[0];
    assert!(
        out1 > 0.0 && out1 < 1.0,
        "first tick integrator output should be in (0,1)"
    );
    assert!(
        out2 > out1 && out2 < 1.0,
        "second tick should retain state and move closer to 1.0; got {out1} -> {out2}"
    );

    let creature = sim.creatures.get(target).expect("target creature");
    assert!(
        !creature.graph_runtime.node_state.is_empty(),
        "graph_runtime.node_state should persist on creature"
    );
    assert!(
        creature.graph_runtime.node_state[0][1] > out1,
        "stored DecayIntegrator state should increase across ticks"
    );
}
