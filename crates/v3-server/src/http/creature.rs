use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use slotmap::{Key, KeyData};
use std::collections::{BTreeMap, BTreeSet};
use v3_core::contracts::ActionOutcome;
use v3_core::contracts::OUTPUT_SLOT_COUNT;
use v3_core::contracts::{CreatureId, InputReference};
use v3_core::creature::action_log::{ActionLogEntry, ActionType};
use v3_core::creature::colour::channels_to_rgb;
use v3_core::genome::cgp::OutputSinkKind;
use v3_core::genome::mesh_annotations::{
    derive_mesh_annotations_with_reachable_indices, MeshNodeAnnotation, MeshReadClass,
    MeshWriteClass,
};
use v3_core::genome::{BackendDef, VmInstruction};
use v3_core::sensors::static_inputs::assemble_static_inputs;
use v3_core::simulation::recording::ActiveTrace;

use crate::error::{AppError, FieldError};
use crate::state::{AppState, SimulationStatus};
use crate::transport::action_log::ActionLogEntry as WireActionLogEntry;
use crate::transport::brain::{brain_view, learning_view, BrainView, NodeLearning};
use crate::transport::sample_assembler::assemble_execution_sample;
use crate::transport::sample_protocol::ExecutionSamplePayload;
use crate::types::PROTOCOL_VERSION;

#[derive(Debug, Default, serde::Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CreatureQuery {
    pub since_tick: Option<u64>,
    pub exclude: Option<String>,
}

#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct CreaturePositionResponse {
    x: u16,
    y: u16,
}

#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct CreaturePhenotypeResponse {
    channels: [u8; 6],
    active_channel: usize,
    polarity: [bool; 6],
    rgb: [u8; 3],
}

#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct CreatureDetailResponse<'a> {
    protocol_version: &'static str,
    id: u64,
    position: CreaturePositionResponse,
    energy: f32,
    /// The creature's own energy capacity (`body.capacity`, T26.F02): the
    /// cap every credit is clamped at, not the configured `max_energy`.
    max_energy: f32,
    /// The stored heritable shares of a steal and a gift (T27.F03), in the
    /// range in force at the creature's birth.
    steal_share: f32,
    give_share: f32,
    age: u64,
    generation: u64,
    /// The direction of the creature's last committed `Move` (T11.F30).
    heading: v3_core::contracts::Direction,
    complexity: u32,
    genome_size: u32,
    phenotype: CreaturePhenotypeResponse,
    /// Tick of the most recent action-log entry, 0 for an empty log.
    latest_tick: u64,
    /// The brain as the simulation applies it (T30.F01); excluded by the
    /// `brain` token.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    brain: Option<BrainView>,
    /// The weights in force of every dynamic edge and the learned scales,
    /// refreshed on every poll; excluded by the `learning` token and omitted
    /// when no node carries a dynamic edge.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    learning: Option<Vec<NodeLearning>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    shared_memory: Option<&'a [f32]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    action_log: Option<Vec<WireActionLogEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    diagnostics: Option<CreatureDiagnosticsResponse>,
}

#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct CreatureDiagnosticsResponse {
    current_inputs: CreatureCurrentInputsDiagnostics,
    live_circuit: CreatureLiveCircuitDiagnostics,
    recent_actions: CreatureRecentActionsDiagnostics,
}

#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct CreatureCurrentInputsDiagnostics {
    food_here: f32,
    neighbor_food: [f32; 8],
    neighbor_barrier: [f32; 8],
    neighbor_occupied: [f32; 8],
}

#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct CreatureLiveCircuitDiagnostics {
    reachable_node_count: usize,
    stateful_reachable_node_count: usize,
    barrier_reader_reachable_node_count: usize,
    barrier_decision_writer_reachable_node_count: usize,
    barrier_reader_without_decision_writer_reachable_node_count: usize,
    distinct_upstream_slots_read: Vec<usize>,
    distinct_payload_slots_written: Vec<usize>,
    distinct_custom_output_slots_written: Vec<usize>,
    reachable_read_class_counts: BTreeMap<String, u32>,
    reachable_write_class_counts: BTreeMap<String, u32>,
}

#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct CreatureRecentActionsDiagnostics {
    sampled_entries: usize,
    blocked_move_count: u32,
    invalid_target_reproduce_count: u32,
    by_action_result: BTreeMap<String, u32>,
}

/// The status of a sample that has started recording.
#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub(crate) enum StartedSampleStatus {
    Recording,
}

#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct StartSampleResponse {
    protocol_version: &'static str,
    status: StartedSampleStatus,
    ticks_requested: u32,
    include_perception_debug: bool,
}

/// A creature's sample, by status: none recording, recording, or complete
/// (handed over once).
#[derive(serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum SampleResponse {
    Idle {
        protocol_version: &'static str,
    },
    Recording {
        protocol_version: &'static str,
        ticks_completed: usize,
        ticks_remaining: u32,
    },
    Complete {
        protocol_version: &'static str,
        sample: ExecutionSamplePayload,
    },
}

fn to_json_value<T: serde::Serialize>(value: T) -> Result<Json<serde_json::Value>, AppError> {
    let body = serde_json::to_value(value)
        .map_err(|error| AppError::Internal(format!("failed to serialize response: {error}")))?;
    Ok(Json(body))
}

fn creature_id_from_ffi_exact(id: u64) -> Option<CreatureId> {
    let key_data = KeyData::from_ffi(id);
    if key_data.as_ffi() != id {
        return None;
    }
    Some(key_data.into())
}

/// Diagnostics map key for a fieldless enum, taken from its serde name.
///
/// Keying the count maps by `String` keeps the JSON object keys sorted
/// alphabetically, which is the order clients already receive.
fn serialized_key<T: serde::Serialize>(value: T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|name| name.as_str().map(str::to_owned))
        .expect("fieldless enum serializes to its variant name")
}

fn build_creature_diagnostics(
    sim: &v3_core::simulation::Simulation,
    creature: &v3_core::creature::state::CreatureState,
    mesh_annotations: &[MeshNodeAnnotation],
    action_log_entries: &[&ActionLogEntry],
) -> CreatureDiagnosticsResponse {
    let static_inputs = assemble_static_inputs(
        &sim.world,
        &creature.sensed_body(&sim.config.energy.capacity),
        &sim.config.energy,
    );
    let current_inputs = CreatureCurrentInputsDiagnostics {
        food_here: static_inputs.food_here,
        neighbor_food: static_inputs.neighbor_food,
        neighbor_barrier: static_inputs.neighbor_barrier,
        neighbor_occupied: static_inputs.neighbor_occupied,
    };

    let mut reachable_read_class_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut reachable_write_class_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut stateful_reachable_node_count = 0usize;
    let mut barrier_reader_reachable_node_count = 0usize;
    let mut barrier_decision_writer_reachable_node_count = 0usize;
    let mut barrier_reader_without_decision_writer_reachable_node_count = 0usize;

    for annotation in mesh_annotations
        .iter()
        .filter(|annotation| annotation.reachable)
    {
        if annotation.has_stateful_behavior {
            stateful_reachable_node_count += 1;
        }
        let reads_barrier = annotation.read_classes.contains(&MeshReadClass::Barrier);
        if reads_barrier {
            barrier_reader_reachable_node_count += 1;
            let writes_decision = annotation
                .write_classes
                .iter()
                .any(|class| matches!(class, MeshWriteClass::Action | MeshWriteClass::Route));
            if writes_decision {
                barrier_decision_writer_reachable_node_count += 1;
            } else {
                barrier_reader_without_decision_writer_reachable_node_count += 1;
            }
        }
        for class in &annotation.read_classes {
            *reachable_read_class_counts
                .entry(serialized_key(class))
                .or_insert(0) += 1;
        }
        for class in &annotation.write_classes {
            *reachable_write_class_counts
                .entry(serialized_key(class))
                .or_insert(0) += 1;
        }
    }

    let reachable_node_ids: BTreeSet<_> = mesh_annotations
        .iter()
        .filter(|annotation| annotation.reachable)
        .map(|annotation| annotation.node_id)
        .collect();
    let mut upstream_slots: BTreeSet<usize> = BTreeSet::new();
    let mut payload_slots: BTreeSet<usize> = BTreeSet::new();
    let mut custom_output_slots: BTreeSet<usize> = BTreeSet::new();
    for node in &creature.genome().nodes {
        if !reachable_node_ids.contains(&node.node_id) {
            continue;
        }
        for input_ref in &node.input_refs {
            if let InputReference::UpstreamSlot(slot) = input_ref {
                if *slot < OUTPUT_SLOT_COUNT {
                    upstream_slots.insert(*slot);
                }
            }
        }
        match &node.backend_def {
            BackendDef::Vm(vm) => {
                for instruction in &vm.program {
                    if let VmInstruction::WriteInternalPayload { slot_idx, .. } = instruction {
                        let slot_idx = *slot_idx as usize;
                        if slot_idx < OUTPUT_SLOT_COUNT {
                            payload_slots.insert(slot_idx);
                        }
                    }
                }
            }
            BackendDef::Graph(graph) => {
                for sink in graph.output_sinks() {
                    if sink.inputs.is_empty() {
                        continue;
                    }
                    if let OutputSinkKind::CustomOutput(slot) = sink.kind {
                        custom_output_slots.insert(slot as usize);
                    }
                }
            }
        }
    }

    let recent_window: Vec<_> = action_log_entries.iter().rev().take(32).copied().collect();
    let mut by_action_result = BTreeMap::new();
    let mut blocked_move_count = 0u32;
    let mut invalid_target_reproduce_count = 0u32;
    for entry in &recent_window {
        let key = format!("{}:{}", entry.action_type.as_key(), entry.outcome.as_key());
        *by_action_result.entry(key).or_insert(0) += 1;
        if entry.action_type == ActionType::Move && entry.outcome == ActionOutcome::Blocked {
            blocked_move_count += 1;
        }
        if entry.action_type == ActionType::Reproduce
            && entry.outcome == ActionOutcome::InvalidTarget
        {
            invalid_target_reproduce_count += 1;
        }
    }

    CreatureDiagnosticsResponse {
        current_inputs,
        live_circuit: CreatureLiveCircuitDiagnostics {
            reachable_node_count: reachable_node_ids.len(),
            stateful_reachable_node_count,
            barrier_reader_reachable_node_count,
            barrier_decision_writer_reachable_node_count,
            barrier_reader_without_decision_writer_reachable_node_count,
            distinct_upstream_slots_read: upstream_slots.into_iter().collect(),
            distinct_payload_slots_written: payload_slots.into_iter().collect(),
            distinct_custom_output_slots_written: custom_output_slots.into_iter().collect(),
            reachable_read_class_counts,
            reachable_write_class_counts,
        },
        recent_actions: CreatureRecentActionsDiagnostics {
            sampled_entries: recent_window.len(),
            blocked_move_count,
            invalid_target_reproduce_count,
            by_action_result,
        },
    }
}

pub async fn get_creature(
    State(app): State<AppState>,
    Path(id): Path<u64>,
    Query(query): Query<CreatureQuery>,
) -> Result<impl IntoResponse, AppError> {
    let handle = app.sim.lock().await;
    let sim = &handle.sim;

    let creature_id = creature_id_from_ffi_exact(id)
        .ok_or_else(|| AppError::NotFound(format!("creature {id} not found")))?;
    let creature = sim
        .creatures
        .get(creature_id)
        .ok_or_else(|| AppError::NotFound(format!("creature {id} not found")))?;

    let rgb = channels_to_rgb(creature.body.phenotype.channels);
    let exclude: std::collections::HashSet<&str> = query
        .exclude
        .as_deref()
        .map(|s| s.split(',').map(str::trim).collect())
        .unwrap_or_default();

    let all_action_log_entries: Vec<&ActionLogEntry> = sim
        .action_logs
        .get(creature_id)
        .map(|log| log.entries().iter().collect())
        .unwrap_or_default();

    let action_log_entries: Vec<&ActionLogEntry> = if exclude.contains("action_log") {
        Vec::new()
    } else if let Some(since) = query.since_tick {
        all_action_log_entries
            .iter()
            .copied()
            .filter(|entry| entry.tick > since)
            .collect()
    } else {
        all_action_log_entries.clone()
    };

    let latest_tick = sim
        .action_logs
        .get(creature_id)
        .and_then(|log| log.entries().back().map(|entry| entry.tick))
        .unwrap_or(0);

    let include_brain = !exclude.contains("brain");
    let include_diagnostics = !exclude.contains("diagnostics");
    let mesh_annotations = if include_brain || include_diagnostics {
        derive_mesh_annotations_with_reachable_indices(
            creature.genome(),
            creature.reachable_nodes(),
        )
    } else {
        Vec::new()
    };

    let diagnostics = include_diagnostics.then(|| {
        build_creature_diagnostics(sim, creature, &mesh_annotations, &all_action_log_entries)
    });

    let response = CreatureDetailResponse {
        protocol_version: PROTOCOL_VERSION,
        id,
        position: CreaturePositionResponse {
            x: creature.position.x,
            y: creature.position.y,
        },
        energy: creature.energy,
        max_energy: creature.energy_cap(),
        steal_share: creature.body.steal_share,
        give_share: creature.body.give_share,
        age: creature.age,
        generation: creature.generation,
        heading: creature.heading,
        complexity: creature.complexity(),
        genome_size: creature.genome().genome_size(),
        phenotype: CreaturePhenotypeResponse {
            channels: creature.body.phenotype.channels,
            active_channel: creature.body.phenotype.active_channel,
            polarity: creature.body.phenotype.polarity,
            rgb,
        },
        latest_tick,
        brain: include_brain.then(|| {
            brain_view(
                creature.genome(),
                creature.graph_runtime().learned(),
                &mesh_annotations,
            )
        }),
        learning: if exclude.contains("learning") {
            None
        } else {
            learning_view(creature.genome(), creature.graph_runtime().learned())
        },
        shared_memory: (!exclude.contains("shared_memory")).then(|| &creature.shared_memory[..]),
        action_log: (!exclude.contains("action_log")).then(|| {
            action_log_entries
                .into_iter()
                .map(WireActionLogEntry::from)
                .collect()
        }),
        diagnostics,
    };

    to_json_value(response)
}

#[derive(serde::Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct StartSampleRequest {
    #[serde(default = "default_sample_ticks")]
    ticks: u32,
    #[serde(default)]
    include_perception_debug: bool,
}

fn default_sample_ticks() -> u32 {
    10
}

pub async fn start_sample(
    State(app): State<AppState>,
    Path(id): Path<u64>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, AppError> {
    let req: StartSampleRequest = if body.is_empty() {
        StartSampleRequest {
            ticks: 10,
            include_perception_debug: false,
        }
    } else {
        serde_json::from_slice(&body)
            .map_err(|error| AppError::InvalidRequest(format!("invalid JSON: {error}")))?
    };

    if req.ticks < 1 || req.ticks > 10 {
        return Err(AppError::ValidationRejected {
            field_errors: vec![FieldError {
                field: "ticks".into(),
                reason: "ticks must be between 1 and 10".into(),
            }],
            endpoint: "start_sample",
        });
    }

    let mut handle = app.sim.lock().await;
    if handle.status == SimulationStatus::Idle {
        return Err(AppError::InvalidStateTransition {
            expected: Some("running or paused".into()),
            current: "idle".into(),
        });
    }

    let creature_id = creature_id_from_ffi_exact(id)
        .ok_or_else(|| AppError::NotFound(format!("creature {id} not found")))?;
    if !handle.sim.creatures.contains_key(creature_id) {
        return Err(AppError::NotFound(format!("creature {id} not found")));
    }

    let mut active = ActiveTrace::new(creature_id, req.ticks);
    active.include_perception_debug = req.include_perception_debug;
    #[cfg(feature = "telemetry")]
    {
        let replaced = handle.active_trace.take();
        let (sim, samples) = handle.telemetry_parts();
        app.telemetry
            .start_sample(sim, samples.window, replaced.as_ref(), &active);
        // A removal before the sample's first tick keeps its cause.
        handle.sim.observe_creature(creature_id);
    }
    handle.active_trace = Some(active);

    to_json_value(StartSampleResponse {
        protocol_version: PROTOCOL_VERSION,
        status: StartedSampleStatus::Recording,
        ticks_requested: req.ticks,
        include_perception_debug: req.include_perception_debug,
    })
}

pub async fn get_sample(
    State(app): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let mut handle = app.sim.lock().await;

    let Some(creature_id) = creature_id_from_ffi_exact(id) else {
        return to_json_value(SampleResponse::Idle {
            protocol_version: PROTOCOL_VERSION,
        });
    };
    let Some(active) = handle
        .active_trace
        .as_ref()
        .filter(|trace| trace.creature_id == creature_id)
    else {
        return to_json_value(SampleResponse::Idle {
            protocol_version: PROTOCOL_VERSION,
        });
    };

    if active.is_complete() {
        let ffi_id = active.creature_id.data().as_ffi();
        let trace = handle
            .active_trace
            .take()
            .expect("active_trace confirmed Some above");
        #[cfg(feature = "telemetry")]
        app.telemetry.hand_over(&handle.sim, &trace);
        let sample = assemble_execution_sample(trace.into_sample(ffi_id));

        return to_json_value(SampleResponse::Complete {
            protocol_version: PROTOCOL_VERSION,
            sample,
        });
    }

    to_json_value(SampleResponse::Recording {
        protocol_version: PROTOCOL_VERSION,
        ticks_completed: active.ticks.len(),
        ticks_remaining: active.ticks_remaining,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reachable_class_counts_serialize_in_sorted_key_order() {
        // Arrange: every mesh class present, with distinct counts so a
        // mis-ordered object is visible in the serialized bytes.
        let read_classes = [
            MeshReadClass::Food,
            MeshReadClass::Neighbor,
            MeshReadClass::Barrier,
            MeshReadClass::Occupancy,
            MeshReadClass::Introspection,
            MeshReadClass::Upstream,
            MeshReadClass::ActionQueue,
            MeshReadClass::Decision,
        ];
        let write_classes = [
            MeshWriteClass::Route,
            MeshWriteClass::Action,
            MeshWriteClass::Memory,
            MeshWriteClass::Payload,
        ];
        let live_circuit = CreatureLiveCircuitDiagnostics {
            reachable_node_count: 0,
            stateful_reachable_node_count: 0,
            barrier_reader_reachable_node_count: 0,
            barrier_decision_writer_reachable_node_count: 0,
            barrier_reader_without_decision_writer_reachable_node_count: 0,
            distinct_upstream_slots_read: Vec::new(),
            distinct_payload_slots_written: Vec::new(),
            distinct_custom_output_slots_written: Vec::new(),
            reachable_read_class_counts: read_classes
                .into_iter()
                .zip(1u32..)
                .map(|(class, count)| (serialized_key(class), count))
                .collect(),
            reachable_write_class_counts: write_classes
                .into_iter()
                .zip(1u32..)
                .map(|(class, count)| (serialized_key(class), count))
                .collect(),
        };

        // Act
        let json = serde_json::to_string(&live_circuit).expect("serializes");

        // Assert: keys are emitted alphabetically, not in enum declaration
        // order, so the endpoint bytes stay byte-identical for clients.
        let expected_read = concat!(
            r#""reachable_read_class_counts":{"#,
            r#""action_queue":7,"barrier":3,"decision":8,"food":1,"introspection":5,"#,
            r#""neighbor":2,"occupancy":4,"upstream":6}"#,
        );
        let expected_write = concat!(
            r#""reachable_write_class_counts":{"#,
            r#""action":2,"memory":3,"payload":4,"route":1}"#,
        );
        assert!(json.contains(expected_read), "{json}");
        assert!(json.contains(expected_write), "{json}");
    }

    #[test]
    fn action_log_keys_use_variant_names() {
        assert_eq!(ActionType::Move.as_key(), "Move");
        assert_eq!(ActionOutcome::NothingToTake.as_key(), "NothingToTake");
    }
}
