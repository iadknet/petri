const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";
const BIND_ADDR_ENV: &str = "V3_SERVER_BIND_ADDR";

fn resolve_bind_addr(env_value: Option<String>) -> String {
    match env_value {
        Some(addr) if !addr.trim().is_empty() => addr,
        _ => DEFAULT_BIND_ADDR.to_string(),
    }
}

#[cfg(feature = "telemetry")]
#[derive(clap::Parser)]
#[command(name = "v3-server")]
struct Args {
    /// Export run telemetry over OTLP (`on` or `off`); beats PETRI_TELEMETRY,
    /// default off.
    #[arg(long, value_name = "on|off")]
    telemetry: Option<v3_telemetry::Switch>,
}

/// Resolves when the process receives SIGINT or (on Unix) SIGTERM.
#[cfg(feature = "telemetry")]
async fn termination_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = terminate.recv() => {}
                }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

/// With the `telemetry` feature the binary also takes `--telemetry`, emits the
/// initial run's `run.started`, and on SIGINT or SIGTERM ends the current run
/// (`shutdown`) and runs the bounded shutdown flush before exit.
#[tokio::main]
async fn main() {
    #[cfg(feature = "telemetry")]
    let switch = {
        use clap::Parser as _;
        v3_telemetry::switch_from_env(Args::parse().telemetry).unwrap_or_else(|message| {
            eprintln!("error: {message}");
            std::process::exit(1);
        })
    };
    let bind_addr = resolve_bind_addr(std::env::var(BIND_ADDR_ENV).ok());
    #[cfg(not(feature = "telemetry"))]
    let state = v3_server::state::AppState::new();
    #[cfg(feature = "telemetry")]
    let state = v3_server::state::AppState::from_config_with_telemetry(
        v3_core::config::SimulationConfig::default(),
        0,
        v3_telemetry::Telemetry::start(v3_telemetry::Service::Server, switch),
    );
    let app = v3_server::router(state.clone());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind");
    println!("v3-server listening on {bind_addr}");
    #[cfg(not(feature = "telemetry"))]
    axum::serve(listener, app).await.expect("server error");
    // Open WebSocket sessions would hold a graceful shutdown open, so the
    // signal stops serving outright, as the default signal action did.
    #[cfg(feature = "telemetry")]
    tokio::select! {
        result = axum::serve(listener, app) => result.expect("server error"),
        () = termination_signal() => state.shutdown_telemetry().await,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_bind_addr;

    #[cfg(feature = "telemetry")]
    #[test]
    fn telemetry_flag_beats_environment_which_beats_default_off() {
        use super::Args;
        use clap::Parser as _;
        use v3_telemetry::Switch;
        let parse = |argv: &[&str]| Args::try_parse_from(argv).expect("arguments must parse");
        let telemetry_switch =
            |args: &Args, env: Option<&str>| v3_telemetry::resolve_switch(args.telemetry, env);
        let plain = parse(&["v3-server"]);
        assert_eq!(telemetry_switch(&plain, None), Ok(Switch::Off));
        assert_eq!(telemetry_switch(&plain, Some("on")), Ok(Switch::On));
        let off = parse(&["v3-server", "--telemetry", "off"]);
        assert_eq!(telemetry_switch(&off, Some("on")), Ok(Switch::Off));
        let on = parse(&["v3-server", "--telemetry", "on"]);
        assert_eq!(telemetry_switch(&on, None), Ok(Switch::On));
        assert!(Args::try_parse_from(["v3-server", "--telemetry", "maybe"]).is_err());
    }

    #[test]
    fn uses_default_bind_addr_when_env_missing() {
        assert_eq!(resolve_bind_addr(None), "0.0.0.0:3000");
    }

    #[test]
    fn uses_env_bind_addr_when_set() {
        assert_eq!(
            resolve_bind_addr(Some("0.0.0.0:3100".to_string())),
            "0.0.0.0:3100"
        );
    }
}
