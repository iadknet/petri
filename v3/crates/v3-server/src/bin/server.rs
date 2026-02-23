#[tokio::main]
async fn main() {
    let app = v3_server::router(v3_server::state::AppState::new());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind");
    println!("v3-server listening on :3000");
    axum::serve(listener, app).await.expect("server error");
}
