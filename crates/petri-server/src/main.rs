use std::net::SocketAddr;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

use petri_core::WorldConfig;
use petri_server::{build_router, sim_loop::run_simulation_loop, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let state = AppState::new(42, WorldConfig::default());
    let sim_state = state.clone();
    tokio::spawn(async move {
        if let Err(err) = run_simulation_loop(sim_state).await {
            tracing::error!("simulation loop exited: {err:#}");
        }
    });

    let app = build_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("petri-server listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
