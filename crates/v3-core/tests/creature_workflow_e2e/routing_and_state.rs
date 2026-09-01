use slotmap::SlotMap;
use v3_core::config::{MutationConfig, OrdinaryFoodTypeId};
use v3_core::contracts::{CreatureId, NodeId, Position, RouteTarget, WorldAction};
use v3_core::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use v3_core::creature::genome::{
    BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction,
};
use v3_core::creature::state::CreatureState;
use v3_core::kernel::WorldState;
use v3_core::simulation::Simulation;

use crate::support::{insert_creature, run_one_traced_tick, test_config};

/// Gate-based routing: negative gate score on slot 0 causes slot 1 (score 0.0) to win.
///
/// CGP graph outputs -1.0 to RouterGate(0) → scores[0] = -1.0.
/// Target id_noop (slot 0): effective = 0.0 + (-1.0) = -1.0
/// Target id_eat  (slot 1): effective = 0.0 + 0.0     =  0.0  (wins)
#[test]
fn cgp_negative_gate_routes_to_higher_scoring_target_e2e() {
    let cfg = test_config();
    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());
    let pos = Position::new(2, 2);
    world.set_food(pos, 1.0);

    let id_entry = NodeId::new(0);
    let id_noop = NodeId::new(1);
    let id_eat = NodeId::new(2);

    // CGP graph: Constant(-1.0) → RouterGate(0) sink
    let entry_def = {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(-1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        // Wire RouterGate(0) sink to CN0
        if let Some(sink) = def
            .output_sinks
            .iter_mut()
            .find(|s| s.kind == OutputSinkKind::RouterGate(0))
        {
            sink.inputs.push(GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            });
        }
        def
    };
    let entry = NodeGenome {
        node_id: id_entry,
        input_refs: vec![],
        backend_def: BackendDef::Graph(entry_def),
        targets: vec![
            RouteTarget {
                target_id: id_noop,
                slot: 0,
                gate_bias: 0.0,
            },
            RouteTarget {
                target_id: id_eat,
                slot: 1,
                gate_bias: 0.0,
            },
        ],
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
    // Gate routing: slot 1 (effective 0.0) beats slot 0 (effective -1.0)
    assert_eq!(
        tick.final_actions[0],
        WorldAction::Eat {
            type_idx: OrdinaryFoodTypeId::default(),
        }
    );
    let route = tick.hops[0]
        .route
        .as_ref()
        .expect("entry hop should have a route decision");
    assert_eq!(route.selected_target_idx, 1);
    assert_eq!(route.selected_target_id, id_eat);
    assert_eq!(route.gate_scores.len(), 2);
    // Slot 0: runtime_score = -1.0, effective = -1.0
    assert!((route.gate_scores[0].runtime_score - (-1.0)).abs() < 1e-6);
    assert!((route.gate_scores[0].effective_score - (-1.0)).abs() < 1e-6);
    // Slot 1: runtime_score = 0.0, effective = 0.0
    assert!((route.gate_scores[1].effective_score - 0.0).abs() < 1e-6);
    assert_eq!(
        tick.hops[1].node_id, id_eat,
        "negative gate on slot 0 should cause slot 1 to win"
    );
}

#[test]
fn graph_state_persists_across_ticks_e2e() {
    let mut cfg = test_config();
    cfg.runtime.max_graph_relax_iters = 1;
    cfg.runtime.graph_convergence_stable_passes = 1;

    let mut world = WorldState::new(cfg.world.width, cfg.world.height, cfg.world.edge_mode);
    world.reconfigure_food(cfg.world.food.clone());
    let pos = Position::new(7, 7);

    // CGP graph: Constant(1.0) → DecayIntegrator(0.5) → CustomOutput(0)
    let state_def = {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
        // CN0: Constant(1.0)
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });
        // CN1: DecayIntegrator(0.5) with input from CN0
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::DecayIntegrator(0.5),
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            plasticity: None,
        });
        // Wire CustomOutput(0) sink to CN1
        if let Some(sink) = def
            .output_sinks
            .iter_mut()
            .find(|s| s.kind == OutputSinkKind::CustomOutput(0))
        {
            sink.inputs.push(GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 1.0,
            });
        }
        def
    };
    let genome = CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome {
            node_id: NodeId::new(0),
            input_refs: vec![],
            backend_def: BackendDef::Graph(state_def),
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
