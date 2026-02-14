use sha2::{Digest, Sha256};

use crate::PROTOCOL_VERSION;
use crate::state::{SimulationPhase, SimulationState};
use v2_core::phenotype::FOUNDER_PHENOTYPE_RGB;

#[derive(Clone, Debug)]
pub struct SimulationApi {
    state: SimulationState,
}

impl SimulationApi {
    #[must_use]
    pub fn new() -> Self {
        let mut api = Self {
            state: SimulationState::new_default(),
        };
        let startup = StartupRequest::default();
        api.startup(startup);
        api
    }

    pub fn startup(&mut self, request: StartupRequest) -> StartupResponse {
        let digest = startup_digest(&request);
        self.state.reset(request, digest);
        StartupResponse {
            protocol_version: PROTOCOL_VERSION.to_string(),
            state: self.state.phase_label().to_string(),
            tick: self.state.tick,
            config_digest: self.state.config_digest.clone(),
        }
    }

    pub fn start(&mut self) -> Result<LifecycleResponse, SimulationError> {
        match self.state.phase {
            SimulationPhase::Idle | SimulationPhase::Paused | SimulationPhase::Running => {
                self.state.phase = SimulationPhase::Running;
                Ok(self.lifecycle_response())
            }
        }
    }

    pub fn pause(&mut self) -> Result<LifecycleResponse, SimulationError> {
        match self.state.phase {
            SimulationPhase::Running | SimulationPhase::Paused => {
                self.state.phase = SimulationPhase::Paused;
                Ok(self.lifecycle_response())
            }
            SimulationPhase::Idle => Err(SimulationError::protocol(
                ProtocolErrorCode::InvalidStateTransition,
                "pause requires running state",
                ErrorDetails {
                    endpoint: Some("/v2/simulation/pause".to_string()),
                    field_errors: Vec::new(),
                    expected_state: Some("running".to_string()),
                    current_state: Some("idle".to_string()),
                },
            )),
        }
    }

    pub fn step(&mut self, steps: Option<u16>) -> Result<LifecycleResponse, SimulationError> {
        if self.state.phase != SimulationPhase::Paused {
            return Err(SimulationError::protocol(
                ProtocolErrorCode::InvalidStateTransition,
                "step requires paused state",
                ErrorDetails {
                    endpoint: Some("/v2/simulation/step".to_string()),
                    field_errors: Vec::new(),
                    expected_state: Some("paused".to_string()),
                    current_state: Some(self.state.phase_label().to_string()),
                },
            ));
        }

        let steps = steps.unwrap_or(1);
        if steps == 0 || steps > 1000 {
            return Err(SimulationError::protocol(
                ProtocolErrorCode::InvalidRequest,
                "steps must be in 1..=1000",
                ErrorDetails {
                    endpoint: Some("/v2/simulation/step".to_string()),
                    field_errors: vec![ErrorField {
                        field: "steps".to_string(),
                        reason: "must be in 1..=1000".to_string(),
                    }],
                    expected_state: None,
                    current_state: None,
                },
            ));
        }

        self.state.advance_ticks(steps);
        Ok(self.lifecycle_response())
    }

    #[must_use]
    pub fn tick_running(&mut self) -> bool {
        if self.state.phase != SimulationPhase::Running {
            return false;
        }

        self.state.advance_ticks(1);
        true
    }

    pub fn paint(&mut self, request: PaintRequest) -> Result<PaintResponse, SimulationError> {
        if self.state.phase == SimulationPhase::Running {
            return Err(SimulationError::protocol(
                ProtocolErrorCode::InvalidStateTransition,
                "paint is only allowed while idle or paused",
                ErrorDetails {
                    endpoint: Some("/v2/simulation/world/paint".to_string()),
                    field_errors: Vec::new(),
                    expected_state: Some("idle|paused".to_string()),
                    current_state: Some("running".to_string()),
                },
            ));
        }

        let touched_cells = match request.action {
            PaintAction::ClearAll => self.state.world.clear_all(),
            PaintAction::Stroke => {
                let tool = request.tool.ok_or_else(|| {
                    SimulationError::protocol(
                        ProtocolErrorCode::InvalidRequest,
                        "stroke requires tool",
                        ErrorDetails {
                            endpoint: Some("/v2/simulation/world/paint".to_string()),
                            field_errors: vec![ErrorField {
                                field: "tool".to_string(),
                                reason: "required for stroke".to_string(),
                            }],
                            expected_state: None,
                            current_state: None,
                        },
                    )
                })?;
                let brush_half_extent = request.brush_half_extent.ok_or_else(|| {
                    SimulationError::protocol(
                        ProtocolErrorCode::InvalidRequest,
                        "stroke requires brush_half_extent",
                        ErrorDetails {
                            endpoint: Some("/v2/simulation/world/paint".to_string()),
                            field_errors: vec![ErrorField {
                                field: "brush_half_extent".to_string(),
                                reason: "required for stroke".to_string(),
                            }],
                            expected_state: None,
                            current_state: None,
                        },
                    )
                })?;
                if brush_half_extent > 2 {
                    return Err(SimulationError::protocol(
                        ProtocolErrorCode::InvalidRequest,
                        "brush_half_extent must be 0, 1, or 2",
                        ErrorDetails {
                            endpoint: Some("/v2/simulation/world/paint".to_string()),
                            field_errors: vec![ErrorField {
                                field: "brush_half_extent".to_string(),
                                reason: "must be 0, 1, or 2".to_string(),
                            }],
                            expected_state: None,
                            current_state: None,
                        },
                    ));
                }
                if request.points.is_empty() {
                    return Err(SimulationError::protocol(
                        ProtocolErrorCode::ValidationRejected,
                        "stroke requires at least one point",
                        ErrorDetails {
                            endpoint: Some("/v2/simulation/world/paint".to_string()),
                            field_errors: vec![ErrorField {
                                field: "points".to_string(),
                                reason: "must not be empty".to_string(),
                            }],
                            expected_state: None,
                            current_state: None,
                        },
                    ));
                }
                for point in &request.points {
                    if !self.state.world.in_bounds(point.x, point.y) {
                        return Err(SimulationError::protocol(
                            ProtocolErrorCode::ValidationRejected,
                            "point out of world bounds",
                            ErrorDetails {
                                endpoint: Some("/v2/simulation/world/paint".to_string()),
                                field_errors: vec![ErrorField {
                                    field: "points".to_string(),
                                    reason: "contains out-of-range coordinates".to_string(),
                                }],
                                expected_state: None,
                                current_state: None,
                            },
                        ));
                    }
                }
                self.state
                    .world
                    .apply_stroke(tool, brush_half_extent, &request.points)
            }
        };

        Ok(PaintResponse {
            protocol_version: PROTOCOL_VERSION.to_string(),
            state: self.state.phase_label().to_string(),
            tick: self.state.tick,
            paint_result: PaintResult { touched_cells },
        })
    }

    #[must_use]
    pub fn status(&self) -> StatusResponse {
        StatusResponse {
            protocol_version: PROTOCOL_VERSION.to_string(),
            state: self.state.phase_label().to_string(),
            tick: self.state.tick,
            sensor_radius: self.state.startup.world.sensor_radius,
            health_window_ticks: self.state.health_window_ticks,
            population: self.state.population,
            mean_energy: self.state.mean_energy,
            births_last_window: self.state.births_last_window,
            deaths_last_window: self.state.deaths_last_window,
            last_action_counts: self.state.last_action_counts.clone(),
        }
    }

    #[must_use]
    pub fn frame(&self) -> FrameResponse {
        let mut food = self
            .state
            .world
            .food
            .iter()
            .map(|cell| FoodSnapshot {
                x: cell.x,
                y: cell.y,
                density: 255,
            })
            .collect::<Vec<_>>();
        food.sort_by_key(|cell| (cell.y, cell.x));

        let mut barriers = self
            .state
            .world
            .barriers
            .iter()
            .map(|cell| BarrierSnapshot {
                x: cell.x,
                y: cell.y,
            })
            .collect::<Vec<_>>();
        barriers.sort_by_key(|cell| (cell.y, cell.x));

        FrameResponse {
            protocol_version: PROTOCOL_VERSION.to_string(),
            tick: self.state.tick,
            width: self.state.world.width,
            height: self.state.world.height,
            creatures: creature_snapshots(&self.state),
            food,
            barriers,
        }
    }

    #[must_use]
    pub fn health(&self) -> HealthPayload {
        let population = self.state.population.max(1);
        HealthPayload {
            population,
            genome_node_count_p50: (1 + population / 50) as u16,
            genome_node_count_p90: (2 + population / 25) as u16,
            mean_energy: self.state.mean_energy,
        }
    }

    fn lifecycle_response(&self) -> LifecycleResponse {
        LifecycleResponse {
            protocol_version: PROTOCOL_VERSION.to_string(),
            state: self.state.phase_label().to_string(),
            tick: self.state.tick,
        }
    }
}

impl Default for SimulationApi {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StartupRequest {
    pub seed: u64,
    pub world: StartupWorld,
    pub population: StartupPopulation,
    pub runtime: StartupRuntime,
}

impl Default for StartupRequest {
    fn default() -> Self {
        Self {
            seed: 0,
            world: StartupWorld {
                width: 32,
                height: 24,
                wrap: true,
                sensor_radius: 4,
            },
            population: StartupPopulation {
                initial_creatures: 10,
                max_creatures: 100,
            },
            runtime: StartupRuntime {
                ticks_per_second: 30,
                max_tick_budget_ms: 16,
            },
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StartupWorld {
    pub width: u16,
    pub height: u16,
    pub wrap: bool,
    pub sensor_radius: u16,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StartupPopulation {
    pub initial_creatures: u32,
    pub max_creatures: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StartupRuntime {
    pub ticks_per_second: u16,
    pub max_tick_budget_ms: u16,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct StartupResponse {
    pub protocol_version: String,
    pub state: String,
    pub tick: u64,
    pub config_digest: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct LifecycleResponse {
    pub protocol_version: String,
    pub state: String,
    pub tick: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StatusResponse {
    pub protocol_version: String,
    pub state: String,
    pub tick: u64,
    pub sensor_radius: u16,
    pub health_window_ticks: u16,
    pub population: u32,
    pub mean_energy: f32,
    pub births_last_window: u32,
    pub deaths_last_window: u32,
    pub last_action_counts: ActionCounts,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ActionCounts {
    #[serde(rename = "move")]
    pub r#move: u32,
    pub eat: u32,
    pub reproduce: u32,
    pub inventory_pickup: u32,
    pub inventory_put: u32,
    pub noop: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FrameResponse {
    pub protocol_version: String,
    pub tick: u64,
    pub width: u16,
    pub height: u16,
    pub creatures: Vec<CreatureSnapshot>,
    pub food: Vec<FoodSnapshot>,
    pub barriers: Vec<BarrierSnapshot>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreatureSnapshot {
    pub id: u64,
    pub x: u16,
    pub y: u16,
    pub energy: f32,
    pub phenotype_rgb: [u8; 3],
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FoodSnapshot {
    pub x: u16,
    pub y: u16,
    pub density: u8,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BarrierSnapshot {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HealthPayload {
    pub population: u32,
    pub genome_node_count_p50: u16,
    pub genome_node_count_p90: u16,
    pub mean_energy: f32,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaintAction {
    Stroke,
    ClearAll,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaintStrokeTool {
    Food,
    Barrier,
    EraseFood,
    EraseBarrier,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct PaintPoint {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PaintRequest {
    pub action: PaintAction,
    pub tool: Option<PaintStrokeTool>,
    pub brush_half_extent: Option<u8>,
    pub points: Vec<PaintPoint>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PaintResponse {
    pub protocol_version: String,
    pub state: String,
    pub tick: u64,
    pub paint_result: PaintResult,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PaintResult {
    pub touched_cells: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolErrorCode {
    InvalidRequest,
    InvalidStateTransition,
    ValidationRejected,
    InternalError,
}

impl ProtocolErrorCode {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::InvalidStateTransition => "invalid_state_transition",
            Self::ValidationRejected => "validation_rejected",
            Self::InternalError => "internal_error",
        }
    }
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct ErrorDetails {
    pub endpoint: Option<String>,
    pub field_errors: Vec<ErrorField>,
    pub expected_state: Option<String>,
    pub current_state: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ErrorField {
    pub field: String,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct ProtocolError {
    pub code: ProtocolErrorCode,
    pub message: String,
    pub details: ErrorDetails,
}

#[derive(Clone, Debug)]
pub enum SimulationError {
    Protocol(Box<ProtocolError>),
}

impl SimulationError {
    #[must_use]
    pub fn protocol(
        code: ProtocolErrorCode,
        message: impl Into<String>,
        details: ErrorDetails,
    ) -> Self {
        Self::Protocol(Box::new(ProtocolError {
            code,
            message: message.into(),
            details,
        }))
    }

    #[must_use]
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Protocol(err) => match err.code {
                ProtocolErrorCode::InvalidRequest => 400,
                ProtocolErrorCode::InvalidStateTransition => 409,
                ProtocolErrorCode::ValidationRejected => 422,
                ProtocolErrorCode::InternalError => 500,
            },
        }
    }

    #[must_use]
    pub fn code(&self) -> ProtocolErrorCode {
        match self {
            Self::Protocol(err) => err.code,
        }
    }

    #[must_use]
    pub fn into_envelope(self) -> ErrorEnvelope {
        ErrorEnvelope::new(self)
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ErrorEnvelope {
    pub protocol_version: String,
    pub error: ErrorBody,
}

impl ErrorEnvelope {
    #[must_use]
    pub fn new(error: SimulationError) -> Self {
        let error = match error {
            SimulationError::Protocol(protocol) => ErrorBody {
                code: protocol.code.as_str().to_string(),
                message: protocol.message,
                details: protocol.details,
            },
        };
        Self {
            protocol_version: PROTOCOL_VERSION.to_string(),
            error,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub details: ErrorDetails,
}

fn creature_snapshots(state: &SimulationState) -> Vec<CreatureSnapshot> {
    if state.world.width == 0 || state.world.height == 0 {
        return Vec::new();
    }

    let mut creatures = Vec::new();
    let max_render = state.population.min(256);
    let tick = state.tick as u32;
    let width = u32::from(state.world.width);
    let height = u32::from(state.world.height);

    for index in 0..max_render {
        let x = ((index.wrapping_mul(31)).wrapping_add(tick.wrapping_mul(7))) % width;
        let y = ((index.wrapping_mul(17)).wrapping_add(tick.wrapping_mul(11))) % height;
        creatures.push(CreatureSnapshot {
            id: u64::from(index) + 1,
            x: x as u16,
            y: y as u16,
            energy: state.mean_energy + (index % 7) as f32 * 0.1,
            phenotype_rgb: FOUNDER_PHENOTYPE_RGB,
        });
    }

    creatures
}

fn startup_digest(request: &StartupRequest) -> String {
    let canonical = canonical_startup_json(request);
    let digest = Sha256::digest(canonical.as_bytes());
    hex::encode(digest)
}

fn canonical_startup_json(request: &StartupRequest) -> String {
    format!(
        "{{\"population\":{{\"initial_creatures\":{},\"max_creatures\":{}}},\"runtime\":{{\"max_tick_budget_ms\":{},\"ticks_per_second\":{}}},\"seed\":{},\"world\":{{\"height\":{},\"sensor_radius\":{},\"width\":{},\"wrap\":{}}}}}",
        request.population.initial_creatures,
        request.population.max_creatures,
        request.runtime.max_tick_budget_ms,
        request.runtime.ticks_per_second,
        request.seed,
        request.world.height,
        request.world.sensor_radius,
        request.world.width,
        request.world.wrap
    )
}
