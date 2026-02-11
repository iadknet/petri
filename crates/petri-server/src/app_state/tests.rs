    use super::{AppState, StartupDraft};

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
