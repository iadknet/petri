use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use slotmap::{Key, KeyData};
use std::collections::{BTreeMap, BTreeSet};
use v3_core::contracts::{CreatureId, InputReference};
use v3_core::creature::action_log::{ActionLogEntry, ActionResult, ActionType};
use v3_core::creature::genome::cgp::OutputSinkKind;
use v3_core::creature::genome::mesh_annotations::{
    derive_mesh_annotations_with_reachable_indices, MeshNodeAnnotation, MeshReadClass,
    MeshWriteClass,
};
use v3_core::creature::genome::{BackendDef, VmInstruction};
use v3_core::mutation::phenotype::channels_to_rgb;
use v3_core::runtime::trace::recording::ActiveTrace;
use v3_core::runtime::OUTPUT_SLOT_COUNT;
use v3_core::sensors::static_inputs::assemble_static_inputs;

use crate::error::{AppError, FieldError};
use crate::state::{AppState, SimulationStatus};
use crate::transport::sample_assembler::assemble_execution_sample;
use crate::transport::sample_protocol::ExecutionSamplePayload;
use crate::types::PROTOCOL_VERSION;

#[derive(Debug, Default, serde::Deserialize)]
pub struct CreatureQuery {
    pub since_tick: Option<u64>,
    pub exclude: Option<String>,
}

#[derive(serde::Serialize)]
struct CreaturePositionResponse {
    x: u16,
    y: u16,
}

#[derive(serde::Serialize)]
struct CreaturePhenotypeResponse {
    channels: [u8; 6],
    active_channel: usize,
    polarity: [bool; 6],
    rgb: [u8; 3],
}

#[derive(serde::Serialize)]
struct CreatureDetailResponse<'a> {
    protocol_version: &'static str,
    id: u64,
    position: CreaturePositionResponse,
    energy: f32,
    max_energy: f32,
    reproductive_reserve: f32,
    reproductive_reserve_capacity: f32,
    age: u64,
    generation: u64,
    complexity: u32,
    genome_size: u32,
    phenotype: CreaturePhenotypeResponse,
    latest_tick: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    genome: Option<&'a v3_core::creature::genome::CreatureGenome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mesh_annotations: Option<Vec<MeshNodeAnnotation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shared_memory: Option<&'a [f32]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_log: Option<Vec<&'a ActionLogEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<CreatureDiagnosticsResponse>,
}

#[derive(serde::Serialize)]
struct CreatureDiagnosticsResponse {
    current_inputs: CreatureCurrentInputsDiagnostics,
    live_circuit: CreatureLiveCircuitDiagnostics,
    recent_actions: CreatureRecentActionsDiagnostics,
}

#[derive(serde::Serialize)]
struct CreatureCurrentInputsDiagnostics {
    food_here: f32,
    neighbor_food: [f32; 8],
    neighbor_barrier: [f32; 8],
    neighbor_occupied: [f32; 8],
}

#[derive(serde::Serialize)]
struct CreatureLiveCircuitDiagnostics {
    reachable_node_count: usize,
    stateful_reachable_node_count: usize,
    barrier_reader_reachable_node_count: usize,
    barrier_decision_writer_reachable_node_count: usize,
    barrier_reader_without_decision_writer_reachable_node_count: usize,
    distinct_upstream_slots_read: Vec<usize>,
    distinct_payload_slots_written: Vec<usize>,
    distinct_custom_output_slots_written: Vec<usize>,
    reachable_read_class_counts: BTreeMap<MeshReadClass, u32>,
    reachable_write_class_counts: BTreeMap<MeshWriteClass, u32>,
}

#[derive(serde::Serialize)]
struct CreatureRecentActionsDiagnostics {
    sampled_entries: usize,
    blocked_move_count: u32,
    invalid_target_reproduce_count: u32,
    by_action_result: BTreeMap<String, u32>,
}

#[derive(serde::Serialize)]
struct StartSampleResponse {
    protocol_version: &'static str,
    status: &'static str,
    ticks_requested: u32,
    include_perception_debug: bool,
}

#[derive(serde::Serialize)]
struct SampleIdleResponse {
    protocol_version: &'static str,
    status: &'static str,
}

#[derive(serde::Serialize)]
struct SampleRecordingResponse {
    protocol_version: &'static str,
    status: &'static str,
    ticks_completed: usize,
    ticks_remaining: u32,
}

#[derive(serde::Serialize)]
struct SampleCompleteResponse {
    protocol_version: &'static str,
    status: &'static str,
    sample: ExecutionSamplePayload,
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

fn build_creature_diagnostics(
    sim: &v3_core::simulation::Simulation,
    creature: &v3_core::creature::state::CreatureState,
    mesh_annotations: &[MeshNodeAnnotation],
    action_log_entries: &[&ActionLogEntry],
) -> CreatureDiagnosticsResponse {
    let static_inputs = assemble_static_inputs(&sim.world, creature);
    let current_inputs = CreatureCurrentInputsDiagnostics {
        food_here: static_inputs.food_here,
        neighbor_food: static_inputs.neighbor_food,
        neighbor_barrier: static_inputs.neighbor_barrier,
        neighbor_occupied: static_inputs.neighbor_occupied,
    };

    let mut reachable_read_class_counts: BTreeMap<MeshReadClass, u32> = BTreeMap::new();
    let mut reachable_write_class_counts: BTreeMap<MeshWriteClass, u32> = BTreeMap::new();
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
            *reachable_read_class_counts.entry(*class).or_insert(0) += 1;
        }
        for class in &annotation.write_classes {
            *reachable_write_class_counts.entry(*class).or_insert(0) += 1;
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
    for node in &creature.genome.nodes {
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
                for sink in &graph.output_sinks {
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
        let key = format!("{}:{}", entry.action_type.as_key(), entry.result.as_key());
        *by_action_result.entry(key).or_insert(0) += 1;
        if entry.action_type == ActionType::Move && entry.result == ActionResult::Blocked {
            blocked_move_count += 1;
        }
        if entry.action_type == ActionType::Reproduce && entry.result == ActionResult::InvalidTarget
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

    let rgb = channels_to_rgb(creature.phenotype_channels);
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

    let include_genome = !exclude.contains("genome");
    let include_diagnostics = !exclude.contains("diagnostics");
    let mesh_annotations = if include_genome || include_diagnostics {
        derive_mesh_annotations_with_reachable_indices(
            &creature.genome,
            &creature.cached_reachable_nodes,
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
        max_energy: sim.config.energy.lifecycle.max_energy,
        reproductive_reserve: creature.reproductive_reserve,
        reproductive_reserve_capacity: sim.config.nutrition.reproductive_reserve_capacity,
        age: creature.age,
        generation: creature.generation,
        complexity: creature.cached_complexity,
        genome_size: creature.genome.genome_size(),
        phenotype: CreaturePhenotypeResponse {
            channels: creature.phenotype_channels,
            active_channel: creature.phenotype_active_channel,
            polarity: creature.phenotype_channel_polarity,
            rgb,
        },
        latest_tick,
        genome: include_genome.then_some(&creature.genome),
        mesh_annotations: include_genome.then_some(mesh_annotations),
        shared_memory: (!exclude.contains("shared_memory")).then(|| &creature.shared_memory[..]),
        action_log: (!exclude.contains("action_log")).then_some(action_log_entries),
        diagnostics,
    };

    to_json_value(response)
}

#[derive(serde::Deserialize)]
struct StartSampleRequest {
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
    handle.active_trace = Some(active);

    to_json_value(StartSampleResponse {
        protocol_version: PROTOCOL_VERSION,
        status: "recording",
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
        return to_json_value(SampleIdleResponse {
            protocol_version: PROTOCOL_VERSION,
            status: "idle",
        });
    };
    let Some(active) = handle
        .active_trace
        .as_ref()
        .filter(|trace| trace.creature_id == creature_id)
    else {
        return to_json_value(SampleIdleResponse {
            protocol_version: PROTOCOL_VERSION,
            status: "idle",
        });
    };

    if active.is_complete() {
        let ffi_id = active.creature_id.data().as_ffi();
        let trace = handle
            .active_trace
            .take()
            .expect("active_trace confirmed Some above");
        let sample = assemble_execution_sample(trace.into_sample(ffi_id))
            .map_err(|error| AppError::Internal(format!("failed to assemble sample: {error}")))?;

        return to_json_value(SampleCompleteResponse {
            protocol_version: PROTOCOL_VERSION,
            status: "complete",
            sample,
        });
    }

    to_json_value(SampleRecordingResponse {
        protocol_version: PROTOCOL_VERSION,
        status: "recording",
        ticks_completed: active.ticks.len(),
        ticks_remaining: active.ticks_remaining,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_class_maps_serialize_with_snake_case_keys() {
        let read_counts: BTreeMap<MeshReadClass, u32> = BTreeMap::from([
            (MeshReadClass::Food, 1),
            (MeshReadClass::Neighbor, 2),
            (MeshReadClass::Barrier, 3),
            (MeshReadClass::Occupancy, 4),
            (MeshReadClass::Introspection, 5),
            (MeshReadClass::Upstream, 6),
            (MeshReadClass::ActionQueue, 7),
        ]);
        let write_counts: BTreeMap<MeshWriteClass, u32> = BTreeMap::from([
            (MeshWriteClass::Route, 1),
            (MeshWriteClass::Action, 2),
            (MeshWriteClass::Memory, 3),
            (MeshWriteClass::Payload, 4),
        ]);

        assert_eq!(
            serde_json::to_value(&read_counts).expect("serializes"),
            serde_json::json!({
                "food": 1,
                "neighbor": 2,
                "barrier": 3,
                "occupancy": 4,
                "introspection": 5,
                "upstream": 6,
                "action_queue": 7,
            })
        );
        assert_eq!(
            serde_json::to_value(&write_counts).expect("serializes"),
            serde_json::json!({
                "route": 1,
                "action": 2,
                "memory": 3,
                "payload": 4,
            })
        );
    }

    #[test]
    fn action_log_keys_use_variant_names() {
        assert_eq!(ActionType::Move.as_key(), "Move");
        assert_eq!(ActionResult::AgeConstraints.as_key(), "AgeConstraints");
        assert_eq!(
            ActionResult::NutritionConstraints.as_key(),
            "NutritionConstraints"
        );
    }
}
