use anyhow::Result;
use std::time::Duration;

use crate::AppState;

pub async fn run_simulation_loop(state: AppState) -> Result<()> {
    loop {
        let (frame, delay_ms) = {
            let mut world = state.world.write().await;
            let tps = world.config.ticks_per_second.max(1);
            let delay_ms = (1000 / tps as u64).max(1);

            if world.config.paused {
                (None, delay_ms)
            } else {
                world.tick();
                let frame = world.frame();
                (Some(frame), delay_ms)
            }
        };

        if let Some(frame) = frame {
            if let Ok(bytes) = rmp_serde::to_vec_named(&frame) {
                let _ = state.frames_tx.send(bytes);
            }
        }

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }
}
