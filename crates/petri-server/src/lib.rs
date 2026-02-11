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

    use petri_core::{World, WorldConfig};

    use crate::{build_router, sim_loop::run_single_iteration, AppState};

    async fn read_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

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
    async fn cors_preflight_allows_vite_dev_origin_for_patch_config() {
        let state = AppState::new_for_tests();
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/config")
                    .header("origin", "http://127.0.0.1:5173")
                    .header("access-control-request-method", "PATCH")
                    .header("access-control-request-headers", "content-type")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("access-control-allow-origin"),
            Some(&"http://127.0.0.1:5173".parse().unwrap())
        );
        assert!(response
            .headers()
            .contains_key("access-control-allow-methods"));
        assert!(response
            .headers()
            .contains_key("access-control-allow-headers"));
    }

    #[tokio::test]
    async fn simulation_starts_idle_and_requires_explicit_start() {
        let state = AppState::new_for_tests();
        let app = build_router(state.clone());

        let status_before = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/simulation/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(status_before.status(), StatusCode::OK);
        let status_before_json = read_json(status_before).await;
        assert_eq!(status_before_json["phase"], "idle");
        assert!(status_before_json["run_id"].is_null());
        assert_eq!(status_before_json["startup_viable"], true);

        let (frame, _delay_ms) = run_single_iteration(&state).await;
        assert!(frame.is_none());

        let start_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/simulation/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(start_response.status(), StatusCode::OK);
        let start_json = read_json(start_response).await;
        assert_eq!(start_json["phase"], "running");
        assert!(start_json["run_id"].as_u64().is_some());

        let (frame, _delay_ms) = run_single_iteration(&state).await;
        assert!(frame.is_some());
    }

    #[tokio::test]
    async fn patch_startup_draft_while_running_sets_pending_restart() {
        let state = AppState::new_for_tests();
        let app = build_router(state);

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/simulation/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let patch_payload = json!({
            "initial_creatures": 350
        });

        let patch_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/simulation/startup-draft")
                    .header("content-type", "application/json")
                    .body(Body::from(patch_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(patch_response.status(), StatusCode::OK);
        let patch_json = read_json(patch_response).await;
        assert_eq!(patch_json["initial_creatures"], 350);

        let status_response = app
            .oneshot(
                Request::builder()
                    .uri("/simulation/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status_json = read_json(status_response).await;
        assert_eq!(status_json["pending_restart"], true);
    }

    #[tokio::test]
    async fn patch_config_updates_runtime_values() {
        let state = AppState::new_for_tests();
        let app = build_router(state);

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/simulation/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let patch_payload = json!({
            "paused": true,
            "ticks_per_second": 12,
            "food_spawn_rate": 0.18,
            "food_growth_rate": 0.22,
            "weight_mutation_rate": 0.31,
            "weight_mutation_magnitude": 0.27,
            "logic_node_mutation_rate": 0.06,
            "structural_mutation_rate": 0.14
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

        let cfg_json = read_json(get_response).await;
        assert_eq!(cfg_json["paused"], true);
        assert_eq!(cfg_json["ticks_per_second"], 12);
        assert_eq!(cfg_json["food_spawn_rate"], 0.18);
        assert_eq!(cfg_json["food_growth_rate"], 0.22);
        assert_eq!(cfg_json["weight_mutation_rate"], 0.31);
        assert_eq!(cfg_json["weight_mutation_magnitude"], 0.27);
        assert_eq!(cfg_json["logic_node_mutation_rate"], 0.06);
        assert_eq!(cfg_json["structural_mutation_rate"], 0.14);
    }

    #[tokio::test]
    async fn restart_clears_pending_restart_and_rotates_seed() {
        let state = AppState::new_for_tests();
        let app = build_router(state);

        let start_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/simulation/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let start_json = read_json(start_response).await;
        let seed_before = start_json["seed"].as_u64().unwrap();
        let run_id_before = start_json["run_id"].as_u64().unwrap();

        let patch_payload = json!({
            "initial_food_density": 0.35
        });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/simulation/startup-draft")
                    .header("content-type", "application/json")
                    .body(Body::from(patch_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let restart_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/simulation/restart")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(restart_response.status(), StatusCode::OK);
        let restart_json = read_json(restart_response).await;

        assert_eq!(restart_json["pending_restart"], false);
        assert_eq!(restart_json["phase"], "running");
        assert_ne!(restart_json["seed"].as_u64().unwrap(), seed_before);
        assert!(restart_json["run_id"].as_u64().unwrap() > run_id_before);
    }

    #[tokio::test]
    async fn non_viable_startup_config_is_rejected() {
        let state = AppState::new_for_tests();
        let app = build_router(state);

        let patch_payload = json!({
            "initial_creatures": 1,
            "initial_food_density": 0.0,
            "food_spawn_rate": 0.0,
            "food_growth_rate": 0.0,
            "energy_per_tick_decay": 0.03,
            "energy_per_move": 0.05
        });

        let patch_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/simulation/startup-draft")
                    .header("content-type", "application/json")
                    .body(Body::from(patch_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(patch_response.status(), StatusCode::OK);

        let start_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/simulation/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(start_response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let err_json = read_json(start_response).await;
        assert_eq!(err_json["code"], "non_viable_startup_config");
    }

    #[tokio::test]
    async fn status_reports_non_viable_startup_draft() {
        let state = AppState::new_for_tests();
        let app = build_router(state);

        let patch_payload = json!({
            "initial_creatures": 1,
            "initial_food_density": 0.0,
            "food_spawn_rate": 0.0,
            "food_growth_rate": 0.0,
            "energy_per_tick_decay": 0.03,
            "energy_per_move": 0.05
        });

        let patch_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/simulation/startup-draft")
                    .header("content-type", "application/json")
                    .body(Body::from(patch_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(patch_response.status(), StatusCode::OK);

        let status_response = app
            .oneshot(
                Request::builder()
                    .uri("/simulation/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(status_response.status(), StatusCode::OK);
        let status_json = read_json(status_response).await;
        assert_eq!(status_json["startup_viable"], false);
        assert_eq!(
            status_json["startup_viability_code"],
            "non_viable_startup_config"
        );
    }

    #[test]
    fn frame_payload_size_stays_below_threshold_for_default_world() {
        let world = World::new(WorldConfig::default(), 7);
        let frame = world.frame();
        let payload = rmp_serde::to_vec_named(&frame).expect("frame should serialize");
        assert!(
            payload.len() < 120_000,
            "frame payload unexpectedly large: {} bytes",
            payload.len()
        );
    }
}
