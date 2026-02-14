use crate::PROTOCOL_VERSION;

#[derive(Clone, Debug, serde::Serialize)]
pub struct RunStartedEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub seed: u64,
    pub ticks_requested: u64,
    pub sample_every: u16,
}

impl RunStartedEvent {
    #[must_use]
    pub fn new(seed: u64, ticks_requested: u64, sample_every: u16) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            event_type: "run_started",
            seed,
            ticks_requested,
            sample_every,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ActionCounts {
    #[serde(rename = "move")]
    pub r#move: u32,
    pub eat: u32,
    pub reproduce: u32,
    pub inventory_pickup: u32,
    pub inventory_put: u32,
    pub noop: u32,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct TickSampleEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub tick: u64,
    pub population: u32,
    pub mean_energy: f32,
    pub births_last_window: u32,
    pub deaths_last_window: u32,
    pub last_action_counts: ActionCounts,
}

impl TickSampleEvent {
    #[must_use]
    pub fn new(
        tick: u64,
        population: u32,
        mean_energy: f32,
        births_last_window: u32,
        deaths_last_window: u32,
        last_action_counts: ActionCounts,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            event_type: "tick_sample",
            tick,
            population,
            mean_energy,
            births_last_window,
            deaths_last_window,
            last_action_counts,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct RunCompletedEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub ticks_executed: u64,
    pub final_population: u32,
    pub final_mean_energy: f32,
}

impl RunCompletedEvent {
    #[must_use]
    pub fn new(ticks_executed: u64, final_population: u32, final_mean_energy: f32) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            event_type: "run_completed",
            ticks_executed,
            final_population,
            final_mean_energy,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct AblationStartedEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub seed: u64,
    pub ticks_requested: u64,
    pub presets: Vec<String>,
}

impl AblationStartedEvent {
    #[must_use]
    pub fn new(seed: u64, ticks_requested: u64, presets: Vec<String>) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            event_type: "ablation_started",
            seed,
            ticks_requested,
            presets,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct AblationResultEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub preset: String,
    pub score: f32,
    pub final_population: u32,
    pub final_mean_energy: f32,
}

impl AblationResultEvent {
    #[must_use]
    pub fn new(preset: String, score: f32, final_population: u32, final_mean_energy: f32) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            event_type: "ablation_result",
            preset,
            score,
            final_population,
            final_mean_energy,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct AblationCompletedEvent {
    pub protocol_version: &'static str,
    pub event_type: &'static str,
    pub results_count: u16,
    pub best_preset: String,
}

impl AblationCompletedEvent {
    #[must_use]
    pub fn new(results_count: u16, best_preset: String) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            event_type: "ablation_completed",
            results_count,
            best_preset,
        }
    }
}

#[must_use]
pub fn as_ndjson_line<T: serde::Serialize>(event: &T) -> String {
    serde_json::to_string(event).expect("serializable event")
}
