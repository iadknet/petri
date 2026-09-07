use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::types::PROTOCOL_VERSION;

#[derive(Debug, serde::Serialize)]
pub struct FieldError {
    pub field: String,
    pub reason: String,
}

#[derive(Debug)]
pub enum AppError {
    InvalidRequest(String),
    InvalidStateTransition {
        expected: Option<String>,
        current: String,
    },
    ValidationRejected {
        field_errors: Vec<FieldError>,
        endpoint: &'static str,
    },
    NotFound(String),
    Internal(String),
}

/// Context block of the normative error envelope
/// (`docs/reference/v3-server-api-protocol-spec.md` §6). Every field is
/// optional, and the whole block is omitted when no field applies.
#[derive(Debug, Default, serde::Serialize)]
struct ErrorDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    field_errors: Vec<FieldError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_state: Option<String>,
}

impl ErrorDetails {
    fn is_empty(&self) -> bool {
        self.endpoint.is_none()
            && self.field_errors.is_empty()
            && self.expected_state.is_none()
            && self.current_state.is_none()
    }
}

#[derive(Debug, serde::Serialize)]
struct ErrorPayload {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "ErrorDetails::is_empty")]
    details: ErrorDetails,
}

#[derive(Debug, serde::Serialize)]
struct ErrorEnvelope {
    protocol_version: &'static str,
    error: ErrorPayload,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match self {
            AppError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "invalid_request",
                msg,
                ErrorDetails::default(),
            ),
            AppError::InvalidStateTransition { expected, current } => (
                StatusCode::CONFLICT,
                "invalid_state_transition",
                format!("current state is '{current}'"),
                ErrorDetails {
                    expected_state: expected,
                    current_state: Some(current),
                    ..ErrorDetails::default()
                },
            ),
            AppError::ValidationRejected {
                field_errors,
                endpoint,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_rejected",
                format!("validation failed for {endpoint}"),
                ErrorDetails {
                    endpoint: Some(endpoint),
                    field_errors,
                    ..ErrorDetails::default()
                },
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "not_found",
                msg,
                ErrorDetails::default(),
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg,
                ErrorDetails::default(),
            ),
        };

        let body = ErrorEnvelope {
            protocol_version: PROTOCOL_VERSION,
            error: ErrorPayload {
                code,
                message,
                details,
            },
        };

        (status, axum::Json(body)).into_response()
    }
}
