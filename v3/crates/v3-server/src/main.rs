use std::sync::Arc;
use tokio::sync::RwLock;

use v3_server::state::ServerState;

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(ServerState::new()));

    // Spawn tick loop
    let tick_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            tick_state.write().await.tick();
        }
    });

    let app = v3_server::build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:4000");
    axum::serve(listener, app).await.unwrap();
}
