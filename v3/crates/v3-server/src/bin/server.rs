const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";
const BIND_ADDR_ENV: &str = "V3_SERVER_BIND_ADDR";

fn resolve_bind_addr(env_value: Option<String>) -> String {
    match env_value {
        Some(addr) if !addr.trim().is_empty() => addr,
        _ => DEFAULT_BIND_ADDR.to_string(),
    }
}

#[tokio::main]
async fn main() {
    let bind_addr = resolve_bind_addr(std::env::var(BIND_ADDR_ENV).ok());
    let app = v3_server::router(v3_server::state::AppState::new());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind");
    println!("v3-server listening on {bind_addr}");
    axum::serve(listener, app).await.expect("server error");
}

#[cfg(test)]
mod tests {
    use super::resolve_bind_addr;

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
