use axum::body::Body;
use http_body_util::BodyExt;
use hyper::Request;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;
use v3_server::state::ServerState;

#[tokio::test]
async fn status_endpoint_returns_valid_json() {
    let state = Arc::new(RwLock::new(ServerState::new()));
    let app = v3_server::build_router(state);

    let response = app
        .oneshot(
            Request::get("/v3/simulation/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["tick"], 0);
    assert_eq!(json["running"], false);
    assert_eq!(json["creature_count"], 50);
}

#[tokio::test]
async fn start_makes_simulation_running() {
    let state = Arc::new(RwLock::new(ServerState::new()));
    let app = v3_server::build_router(state.clone());

    // Start simulation
    let response = app
        .oneshot(
            Request::post("/v3/simulation/start")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Verify running state
    let read_state = state.read().await;
    assert!(read_state.running);
}

#[tokio::test]
async fn pause_stops_simulation() {
    let state = Arc::new(RwLock::new(ServerState::new()));

    // Start first
    {
        let mut write_state = state.write().await;
        write_state.running = true;
    }

    let app = v3_server::build_router(state.clone());

    // Pause simulation
    let response = app
        .oneshot(
            Request::post("/v3/simulation/pause")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let read_state = state.read().await;
    assert!(!read_state.running);
}

#[tokio::test]
async fn tick_advances_when_running() {
    let state = Arc::new(RwLock::new(ServerState::new()));

    // Start and manually tick
    {
        let mut write_state = state.write().await;
        write_state.running = true;
        write_state.tick();
    }

    let app = v3_server::build_router(state);

    let response = app
        .oneshot(
            Request::get("/v3/simulation/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["tick"], 1);
    assert_eq!(json["running"], true);
    assert_eq!(json["creature_count"], 50);
}
