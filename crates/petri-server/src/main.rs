use std::net::SocketAddr;

use anyhow::{Context, Result};
use tracing_subscriber::EnvFilter;

use petri_core::WorldConfig;
use petri_server::{build_router, sim_loop::run_simulation_loop, AppState};

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:4000";

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
    let addr = resolve_server_addr(std::env::var("PETRI_SERVER_ADDR").ok().as_deref())?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("petri-server listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn resolve_server_addr(raw: Option<&str>) -> Result<SocketAddr> {
    let value = raw.unwrap_or(DEFAULT_SERVER_ADDR);
    value
        .parse::<SocketAddr>()
        .with_context(|| format!("invalid PETRI_SERVER_ADDR value: {value}"))
}

#[cfg(test)]
mod tests {
    use super::resolve_server_addr;

    #[test]
    fn resolve_server_addr_defaults_to_localhost_4000() {
        let addr = resolve_server_addr(None).expect("default addr should parse");
        assert_eq!(addr.to_string(), "127.0.0.1:4000");
    }

    #[test]
    fn resolve_server_addr_accepts_custom_bind_addr() {
        let addr = resolve_server_addr(Some("127.0.0.1:4100")).expect("custom addr should parse");
        assert_eq!(addr.to_string(), "127.0.0.1:4100");
    }

    #[test]
    fn resolve_server_addr_rejects_invalid_values() {
        let err = resolve_server_addr(Some("not-an-addr"))
            .expect_err("invalid addr should return an error");
        assert!(format!("{err:#}").contains("invalid PETRI_SERVER_ADDR value"));
    }
}
