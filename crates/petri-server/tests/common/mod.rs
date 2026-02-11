use axum::body::to_bytes;

use petri_core::WorldConfig;
use petri_server::{AppState, AppStateOptions};

#[allow(dead_code)]
pub async fn read_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&bytes).expect("response body should be valid json")
}

pub fn fast_state() -> AppState {
    AppState::new_with_options(
        1,
        WorldConfig::default(),
        AppStateOptions {
            viability_probe_enabled: false,
        },
    )
}
