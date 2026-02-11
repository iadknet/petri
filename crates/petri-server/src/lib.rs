pub mod api;
pub mod app_state;
pub mod sim_loop;

pub use api::build_router;
pub use app_state::AppState;

#[cfg(test)]
mod tests {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    use petri_core::WorldConfig;

    use crate::{build_router, AppState};

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let state = AppState::new_for_tests();
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn patch_config_updates_runtime_values() {
        let state = AppState::new_for_tests();
        let app = build_router(state);

        let patch_payload = json!({
            "paused": true,
            "ticks_per_second": 12
        });

        let patch_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/config")
                    .header("content-type", "application/json")
                    .body(Body::from(patch_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(patch_response.status(), StatusCode::OK);

        let get_response = app
            .oneshot(
                Request::builder()
                    .uri("/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        let bytes = to_bytes(get_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let cfg: WorldConfig = serde_json::from_slice(&bytes).unwrap();
        assert!(cfg.paused);
        assert_eq!(cfg.ticks_per_second, 12);
    }
}
