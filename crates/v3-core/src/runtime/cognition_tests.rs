use crate::config::RuntimeConfig;
use crate::contracts::NodeId;
use crate::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource, OutputSinkKind,
};
use crate::genome::{BackendDef, CreatureGenome, NodeGenome, VmBackendDef, VmInstruction};
use crate::runtime::state::GraphRuntimeState;
use crate::runtime::trace::domain::{BackendTrace, TerminationReason};
use crate::runtime::types::{sanitize_f32, MeshOutput};
use crate::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use crate::sensors::static_inputs::StaticInputs;
use crate::sensors::typed_food::TypedFoodLocalSnapshot;
use proptest::prelude::*;

fn genome(backend_def: BackendDef) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: NodeId::new(0),
        nodes: vec![NodeGenome::new(NodeId::new(0), vec![], backend_def, vec![])],
    }
}

fn sensors() -> SensorSnapshot {
    SensorSnapshot {
        world_reads: Default::default(),
        local: StaticInputs {
            food_here: 0.0,
            neighbor_food: [0.0; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            energy_scale: 200.0,
            age_ticks: 0.0,
            own_colour: [0.0; 3],
            previous_outcome: [0.0; 4],
        },
        typed_local_food: TypedFoodLocalSnapshot::zeroed(1),
        perception: PerceptionSnapshot::zeroed(1),
    }
}

fn run_both(
    genome: &CreatureGenome,
    initial: f32,
    energy: f32,
    config: &RuntimeConfig,
) -> (MeshOutput, f32) {
    let mut memory = [0.0; 16];
    memory[0] = initial;
    let mut traced_memory = memory;
    let mut energy_plain = energy;
    let mut energy_traced = energy;
    let mut state =
        GraphRuntimeState::for_genome(genome, &crate::genome::learning::Seeds::default());
    state.begin_tick(0);
    let mut traced_state = state.clone();
    let output = super::execute_creature_mesh(
        genome,
        &sensors(),
        &mut energy_plain,
        &mut memory,
        &[0.0; 16],
        &mut state,
        config,
        crate::contracts::Direction::N,
    );
    let (traced, hops, _) = super::traced_mesh::execute_creature_mesh_traced(
        genome,
        &sensors(),
        &mut energy_traced,
        &mut traced_memory,
        &[0.0; 16],
        &mut traced_state,
        config,
        crate::contracts::Direction::N,
    );
    assert_eq!(output.work_counters, traced.work_counters);
    assert_eq!(output.actions, traced.actions);
    assert_eq!(output.termination_reason, traced.termination_reason);
    assert_eq!(energy_plain, energy_traced);
    assert_eq!(memory, traced_memory);
    assert_eq!(state.learned(), traced_state.learned());
    if matches!(genome.nodes[0].backend_def(), BackendDef::Vm(_)) {
        let writes: usize = hops
            .iter()
            .map(|hop| match &hop.backend_trace {
                BackendTrace::Vm(trace) => trace.slot_writes.len(),
                BackendTrace::Graph(_) => 0,
            })
            .sum();
        assert_eq!(
            output.work_counters.shared_memory_writes_changed as usize,
            writes
        );
    }
    (output, memory[0])
}

fn vm_writes(values: &[f32], clear: bool) -> CreatureGenome {
    let mut program = Vec::new();
    for (index, _) in values.iter().enumerate() {
        program.push(VmInstruction::LoadConst {
            dst: 0,
            const_idx: index as u8,
        });
        program.push(if index % 2 == 0 {
            VmInstruction::StoreSlotImm {
                slot_idx: 0,
                src: 0,
            }
        } else {
            VmInstruction::StoreSlot {
                slot_reg: 1,
                src: 0,
            }
        });
    }
    if clear {
        program.push(VmInstruction::ClearSlot { slot_idx: 0 });
    }
    genome(BackendDef::Vm(VmBackendDef {
        register_count: 2,
        constants: values.to_vec(),
        program,
    }))
}

/// A genome writing its values to slot 0 then clearing it when asked.
type Writes = fn(&[f32], bool) -> CreatureGenome;

/// The values a Graph pass can write to slot 0: its catalog holds one
/// `WriteSlot(0)` sink, so at most the first.
fn graph_expressible(values: &[f32]) -> &[f32] {
    &values[..values.len().min(1)]
}

/// A Graph writing `values` (at most one, [`graph_expressible`]) to slot 0
/// through its `WriteSlot(0)` sink, then clearing it through `ClearSlot(0)`,
/// which follows in catalog order.
fn graph_writes(values: &[f32], clear: bool) -> CreatureGenome {
    assert!(values.len() <= 1, "the catalog holds one WriteSlot(0) sink");
    let edge = |weight| GraphEdge::new(GraphSource::ComputeNode(0), weight);
    let def = CgpGraphBackendDef::new_with_fixed_outputs()
        .with_compute_nodes(vec![ComputeNode::new(
            ComputeNodeKind::Constant(1.0),
            vec![],
        )])
        .with_sink_inputs(
            OutputSinkKind::WriteSlot(0),
            values.iter().map(|&value| edge(value)).collect(),
        );
    genome(BackendDef::Graph(if clear {
        def.with_sink_inputs(OutputSinkKind::ClearSlot(0), vec![edge(1.0)])
    } else {
        def
    }))
}

#[test]
fn memory_counts_each_changed_store_and_clear_with_traced_parity() {
    // VM: initial 3 -> 2 -> 2 -> 3 -> 0: the repeated write is unchanged,
    // and reversing back to 3 still counts before the final clear. Graph:
    // 3 -> 2 -> 0, its one write and its clear.
    let cases: [(Writes, &[f32], u32); 2] =
        [(vm_writes, &[2.0, 2.0, 3.0], 3), (graph_writes, &[2.0], 2)];
    for (build, values, changed) in cases {
        let (output, final_value) =
            run_both(&build(values, true), 3.0, 100.0, &RuntimeConfig::default());
        assert_eq!(output.work_counters.shared_memory_writes_changed, changed);
        assert_eq!(final_value, 0.0);
    }
    for build in [vm_writes, graph_writes] {
        let (output, _) = run_both(&build(&[0.0], true), 0.0, 100.0, &RuntimeConfig::default());
        assert_eq!(output.work_counters.shared_memory_writes_changed, 0);
    }
}

#[test]
fn memory_changed_write_preserves_epsilon_boundary_and_sanitizes_values() {
    for build in [vm_writes, graph_writes] {
        for (initial, value, expected) in [
            (0.0, f32::EPSILON / 2.0, 0),
            (0.0, f32::EPSILON, 0),
            (0.0, f32::EPSILON * 2.0, 1),
            (0.0, f32::NAN, 0),
            (1.0e9, f32::INFINITY, 0),
            (0.0, f32::NEG_INFINITY, 1),
        ] {
            let (output, final_value) = run_both(
                &build(&[value], false),
                initial,
                100.0,
                &RuntimeConfig::default(),
            );
            assert_eq!(output.work_counters.shared_memory_writes_changed, expected);
            assert_eq!(final_value, sanitize_f32(value));
        }
    }
}

#[test]
fn graph_unwired_sinks_do_not_count_as_writes() {
    let g = graph_writes(&[], false);
    let (output, final_value) = run_both(&g, 3.0, 100.0, &RuntimeConfig::default());
    assert_eq!(output.work_counters.shared_memory_writes_changed, 0);
    assert_eq!(final_value, 3.0);
}

#[test]
fn vm_counts_executed_writes_before_exhaustion_but_not_the_unaffordable_write() {
    let g = vm_writes(&[1.0, 2.0], false);
    let mut config = RuntimeConfig::default();
    config.vm.opcode_cost_multiplier = 1.0;
    config.vm.step_ramp_cost = 0.0;
    // LoadConst .08, StoreSlotImm .12, LoadConst .08, StoreSlot .14.
    let (output, final_value) = run_both(&g, 0.0, 0.3, &config);
    assert_eq!(
        output.termination_reason,
        TerminationReason::EnergyExhausted
    );
    assert_eq!(output.work_counters.vm_steps, 3);
    assert_eq!(output.work_counters.shared_memory_writes_changed, 1);
    assert_eq!(
        final_value, 0.0,
        "exhaustion still discards the working copy"
    );

    let mut stopped = vm_writes(&[1.0], false);
    let BackendDef::Vm(def) = stopped.nodes[0].backend_edit().1 else {
        unreachable!()
    };
    def.program.insert(0, VmInstruction::Halt);
    let (output, _) = run_both(&stopped, 0.0, 100.0, &config);
    assert_eq!(output.work_counters.shared_memory_writes_changed, 0);
}

proptest! {
    #[test]
    fn memory_event_counts_follow_each_immediate_predecessor(
        initial in -10.0f32..10.0, values in prop::collection::vec(any::<f32>(), 0..12), clear in any::<bool>(),
    ) {
        let cases: [(Writes, &[f32]); 2] =
            [(vm_writes, &values), (graph_writes, graph_expressible(&values))];
        for (build, values) in cases {
            let mut previous = initial;
            let mut expected = 0;
            for next in values.iter().copied().map(sanitize_f32).chain(clear.then_some(0.0)) {
                expected += u32::from((previous - next).abs() > f32::EPSILON);
                previous = next;
            }
            let (output, final_value) = run_both(&build(values, clear), initial, 100.0, &RuntimeConfig::default());
            prop_assert_eq!(output.work_counters.shared_memory_writes_changed, expected);
            prop_assert!(expected <= values.len() as u32 + u32::from(clear));
            prop_assert_eq!(final_value, previous);
        }
    }
}
