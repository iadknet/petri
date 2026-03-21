//! CGP graph backend mesh annotation derivation.
//!
//! Derives read/write classes and stateful flags from the CGP three-layer model:
//! - Read classes from `GraphSource::InputLeaf` ref_idx in edges
//! - Write classes from `OutputSinkKind` + `ActionSlotBehavior`
//! - Terminal class from `ExecuteGate`
//! - Stateful from `ComputeNodeKind` + plasticity

use std::collections::BTreeSet;

use crate::contracts::InputReference;
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNodeKind, GraphSource, OutputSinkKind,
};
use crate::creature::genome::mesh_annotations::{MeshReadClass, MeshWriteClass};

use super::cgp_analysis::cgp_live_compute_indices;

/// Derive read/write/stateful annotations for a CGP graph backend.
///
/// Returns (read_classes, write_classes, has_stateful_behavior, live_compute_indices).
pub(crate) fn derive_cgp_annotations(
    def: &CgpGraphBackendDef,
    input_refs: &[InputReference],
) -> (Vec<MeshReadClass>, Vec<MeshWriteClass>, bool, Vec<usize>) {
    let mut read_classes = BTreeSet::new();
    let mut write_classes = BTreeSet::new();
    let mut has_stateful_behavior = false;

    let live_indices = cgp_live_compute_indices(def);

    // Stateful and read classes from live compute nodes
    for &idx in &live_indices {
        let node = &def.compute_nodes[idx];
        if node.plasticity.is_some() {
            has_stateful_behavior = true;
        }
        match &node.kind {
            ComputeNodeKind::DecayIntegrator(_)
            | ComputeNodeKind::Momentum(_)
            | ComputeNodeKind::Oscillator(_)
            | ComputeNodeKind::AdaptiveGain => {
                has_stateful_behavior = true;
            }
            _ => {}
        }

        // Read classes from InputLeaf edges on live compute nodes
        for edge in &node.inputs {
            if let GraphSource::InputLeaf { ref_idx, .. } = edge.source {
                if let Some(input_ref) = input_refs.get(ref_idx as usize) {
                    read_classes.insert(classify_input_ref(input_ref));
                }
            }
            if let GraphSource::SharedMemory { .. } = edge.source {
                has_stateful_behavior = true;
            }
        }
    }

    // Write classes from wired output sinks
    for sink in &def.output_sinks {
        if sink.inputs.is_empty() {
            continue;
        }
        match &sink.kind {
            OutputSinkKind::CustomOutput(_) => {
                write_classes.insert(MeshWriteClass::Payload);
            }
            OutputSinkKind::RouterGate(_) => {
                write_classes.insert(MeshWriteClass::Route);
            }
            OutputSinkKind::WriteSlot(_) | OutputSinkKind::ClearSlot(_) => {
                has_stateful_behavior = true;
                write_classes.insert(MeshWriteClass::Memory);
            }
        }

        // Read classes from InputLeaf edges on sinks
        for edge in &sink.inputs {
            if let GraphSource::InputLeaf { ref_idx, .. } = edge.source {
                if let Some(input_ref) = input_refs.get(ref_idx as usize) {
                    read_classes.insert(classify_input_ref(input_ref));
                }
            }
        }
    }

    // Write classes from wired action bank
    for slot in &def.action_bank {
        let wired = !slot.gate_inputs.is_empty() || !slot.param_inputs.is_empty();
        if !wired {
            continue;
        }
        write_classes.insert(MeshWriteClass::Action);

        // Read classes from action slot edges
        for edge in slot.gate_inputs.iter().chain(slot.param_inputs.iter()) {
            if let GraphSource::InputLeaf { ref_idx, .. } = edge.source {
                if let Some(input_ref) = input_refs.get(ref_idx as usize) {
                    read_classes.insert(classify_input_ref(input_ref));
                }
            }
        }
    }

    // Write class from wired execute gate (terminal = action)
    if !def.execute_gate.inputs.is_empty() {
        write_classes.insert(MeshWriteClass::Action);

        for edge in &def.execute_gate.inputs {
            if let GraphSource::InputLeaf { ref_idx, .. } = edge.source {
                if let Some(input_ref) = input_refs.get(ref_idx as usize) {
                    read_classes.insert(classify_input_ref(input_ref));
                }
            }
        }
    }

    (
        read_classes.into_iter().collect(),
        write_classes.into_iter().collect(),
        has_stateful_behavior,
        live_indices,
    )
}

fn classify_input_ref(input_ref: &InputReference) -> MeshReadClass {
    use crate::contracts::WorldInputKey;

    match input_ref {
        InputReference::World(key) => match key {
            WorldInputKey::FoodHere { .. } | WorldInputKey::AreaFoodSummary { .. } => {
                MeshReadClass::Food
            }
            WorldInputKey::NeighborFoodRing { .. } => MeshReadClass::Food,
            WorldInputKey::NeighborBarrierRing | WorldInputKey::AreaBarrierSummary => {
                MeshReadClass::Barrier
            }
            WorldInputKey::NeighborOccupiedRing | WorldInputKey::AreaOccupancySummary => {
                MeshReadClass::Occupancy
            }
            WorldInputKey::NearbyCreatureCore
            | WorldInputKey::NearbyCreatureVitals
            | WorldInputKey::NearbyCreatureIdentity => MeshReadClass::Neighbor,
        },
        InputReference::StaticIntrospection(_) | InputReference::DynamicIntrospection(_) => {
            MeshReadClass::Introspection
        }
        InputReference::UpstreamSlot(_) => MeshReadClass::Upstream,
        InputReference::ActionQueue => MeshReadClass::ActionQueue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrdinaryFoodTypeId;
    use crate::contracts::WorldInputKey;
    use crate::creature::genome::cgp::{
        ActionSlot, ActionSlotBehavior, ComputeNode, ComputeNodeKind, ExecuteGate, GraphEdge,
        OutputSink, OutputSinkKind, WorldActionKind,
    };

    #[test]
    fn empty_graph_no_annotations() {
        let def = CgpGraphBackendDef {
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let (reads, writes, stateful, live) = derive_cgp_annotations(&def, &[]);
        assert!(reads.is_empty());
        assert!(writes.is_empty());
        assert!(!stateful);
        assert!(live.is_empty());
    }

    #[test]
    fn wired_sink_produces_write_class() {
        let def = CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    weight: 1.0,
                }],
                plasticity: None,
            }],
            output_sinks: vec![
                OutputSink {
                    kind: OutputSinkKind::CustomOutput(0),
                    inputs: vec![GraphEdge {
                        source: GraphSource::ComputeNode(0),
                        weight: 1.0,
                    }],
                },
                OutputSink {
                    kind: OutputSinkKind::RouterGate(0),
                    inputs: Vec::new(), // unwired — should not count
                },
            ],
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let input_refs = vec![InputReference::World(WorldInputKey::FoodHere {
            type_idx: OrdinaryFoodTypeId::default(),
        })];
        let (reads, writes, _, live) = derive_cgp_annotations(&def, &input_refs);

        assert!(writes.contains(&MeshWriteClass::Payload));
        assert!(!writes.contains(&MeshWriteClass::Route)); // unwired router
        assert!(reads.contains(&MeshReadClass::Food));
        assert!(live.contains(&0));
    }

    #[test]
    fn stateful_node_detected() {
        let def = CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::DecayIntegrator(0.5),
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: vec![OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
            }],
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let (_, _, stateful, _) = derive_cgp_annotations(&def, &[]);
        assert!(stateful);
    }

    #[test]
    fn action_slot_produces_action_write() {
        let def = CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Constant(1.0),
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: Vec::new(),
            action_bank: vec![ActionSlot {
                behavior: ActionSlotBehavior::Emit(WorldActionKind::Eat),
                gate_inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
                param_inputs: Vec::new(),
            }],
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let (_, writes, _, _) = derive_cgp_annotations(&def, &[]);
        assert!(writes.contains(&MeshWriteClass::Action));
    }

    #[test]
    fn memory_sink_is_stateful() {
        let def = CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Constant(1.0),
                inputs: Vec::new(),
                plasticity: None,
            }],
            output_sinks: vec![OutputSink {
                kind: OutputSinkKind::WriteSlot(0),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
            }],
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let (_, writes, stateful, _) = derive_cgp_annotations(&def, &[]);
        assert!(stateful);
        assert!(writes.contains(&MeshWriteClass::Memory));
    }

    #[test]
    fn shared_memory_source_is_stateful() {
        let def = CgpGraphBackendDef {
            compute_nodes: vec![ComputeNode {
                kind: ComputeNodeKind::Add,
                inputs: vec![GraphEdge {
                    source: GraphSource::SharedMemory {
                        slot: 0,
                        previous: false,
                    },
                    weight: 1.0,
                }],
                plasticity: None,
            }],
            output_sinks: vec![OutputSink {
                kind: OutputSinkKind::CustomOutput(0),
                inputs: vec![GraphEdge {
                    source: GraphSource::ComputeNode(0),
                    weight: 1.0,
                }],
            }],
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        };
        let (_, _, stateful, _) = derive_cgp_annotations(&def, &[]);
        assert!(stateful);
    }

    #[test]
    fn read_class_from_action_slot_and_execute_gate_edges() {
        let def = CgpGraphBackendDef {
            compute_nodes: Vec::new(),
            output_sinks: Vec::new(),
            action_bank: vec![ActionSlot {
                behavior: ActionSlotBehavior::Emit(WorldActionKind::Move),
                gate_inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 0,
                        sub_idx: 0,
                    },
                    weight: 1.0,
                }],
                param_inputs: Vec::new(),
            }],
            execute_gate: ExecuteGate {
                inputs: vec![GraphEdge {
                    source: GraphSource::InputLeaf {
                        ref_idx: 1,
                        sub_idx: 0,
                    },
                    weight: 1.0,
                }],
            },
        };
        let input_refs = vec![
            InputReference::World(WorldInputKey::FoodHere {
                type_idx: OrdinaryFoodTypeId::default(),
            }),
            InputReference::World(WorldInputKey::NeighborOccupiedRing),
        ];
        let (reads, writes, _, _) = derive_cgp_annotations(&def, &input_refs);

        assert!(reads.contains(&MeshReadClass::Food));
        assert!(reads.contains(&MeshReadClass::Occupancy));
        assert!(writes.contains(&MeshWriteClass::Action));
    }
}
