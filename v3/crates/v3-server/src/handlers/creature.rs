use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use slotmap::KeyData;
use v3_core::contracts::CreatureId;
use v3_core::mutation::phenotype::channels_to_rgb;

use crate::error::AppError;
use crate::state::AppState;
use crate::types::PROTOCOL_VERSION;

pub async fn get_creature(
    State(app): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let handle = app.sim.lock().await;
    let sim = &handle.sim;

    let creature_id: CreatureId = KeyData::from_ffi(id).into();
    let creature = sim
        .creatures
        .get(creature_id)
        .ok_or_else(|| AppError::NotFound(format!("creature {id} not found")))?;

    let rgb = channels_to_rgb(creature.phenotype_channels);
    let memory: &[u8] = &creature.memory;

    Ok(Json(serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "id": id,
        "position": { "x": creature.position.x, "y": creature.position.y },
        "energy": creature.energy,
        "max_energy": sim.config.energy.lifecycle.initial_energy,
        "age": creature.age,
        "generation": creature.generation,
        "complexity": creature.genome.complexity(),
        "phenotype": {
            "channels": creature.phenotype_channels,
            "active_channel": creature.phenotype_active_channel,
            "polarity": creature.phenotype_channel_polarity,
            "rgb": rgb,
        },
        "genome": creature.genome,
        "memory": memory,
    })))
}
