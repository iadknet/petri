use super::{AppState, StartupDraft};
use super::{SimulationPhase, WorldPaintAction, WorldPaintRequest};
use petri_core::{PaintPoint, PaintTool};

#[tokio::test]
async fn new_for_tests_keeps_viability_probe_enabled() {
    let state = AppState::new_for_tests();
    let status = state.simulation_status().await;

    assert!(status.viability_probe_enabled);
}

#[tokio::test]
async fn new_for_tests_fast_disables_viability_probe() {
    let state = AppState::new_for_tests_fast();
    let status = state.simulation_status().await;

    assert!(!status.viability_probe_enabled);
    assert!(status.startup_viable);
}

#[test]
fn startup_draft_validation_allows_expanded_upper_bounds() {
    let mut draft = StartupDraft::viable_default();
    draft.initial_creatures = 12_000;
    draft.max_creatures = 450_000;
    draft.width = 1_400;
    draft.height = 1_400;
    draft.energy_initial = 12.0;
    draft.energy_per_tick_decay = 0.25;
    draft.energy_per_move = 0.25;

    assert!(draft.validate().is_ok());
}

#[tokio::test]
async fn paint_world_rejects_starting_phase() {
    let state = AppState::new_for_tests_fast();
    {
        let mut sim = state.simulation.write().await;
        sim.phase = SimulationPhase::Starting;
    }

    let err = state
        .paint_world(WorldPaintRequest {
            action: WorldPaintAction::Stroke,
            tool: Some(PaintTool::Food),
            brush_half_extent: Some(0),
            points: vec![PaintPoint { x: 1, y: 1 }],
            idle_preview_mode: None,
        })
        .await
        .expect_err("paint should be rejected while starting");
    assert_eq!(err.code(), "paint_phase_not_editable");
}
