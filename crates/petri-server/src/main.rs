use std::net::SocketAddr;

use anyhow::{Context, Result};
use clap::Parser;
use tracing_subscriber::EnvFilter;

use petri_core::WorldConfig;
use petri_server::{build_router, sim_loop::run_simulation_loop, AppState, AppStateOptions};

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:4000";

#[derive(Debug, Parser)]
#[command(name = "petri-server", about = "Petri simulation server")]
struct Args {
    #[arg(long, default_value_t = false)]
    disable_viability_probe: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args(std::env::args());

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let state = AppState::new_with_options(
        42,
        WorldConfig::default(),
        AppStateOptions {
            viability_probe_enabled: !args.disable_viability_probe,
        },
    );
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

fn parse_args<I, T>(iter: I) -> Args
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    Args::parse_from(iter)
}

#[cfg(test)]
mod tests {
    use super::{parse_args, resolve_server_addr};

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

    #[test]
    fn parse_args_supports_disabling_viability_probe() {
        let args = parse_args(["petri-server", "--disable-viability-probe"]);
        assert!(args.disable_viability_probe);
    }
}
