use anyhow::Result;
use std::time::Duration;

use crate::AppState;

pub async fn run_single_iteration(state: &AppState) -> (Option<Vec<u8>>, u64) {
    state.run_single_iteration().await
}

pub async fn run_simulation_loop(state: AppState) -> Result<()> {
    loop {
        let (frame, delay_ms) = run_single_iteration(&state).await;

        if let Some(bytes) = frame {
            let _ = state.frames_tx.send(bytes);
        }

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }
}
