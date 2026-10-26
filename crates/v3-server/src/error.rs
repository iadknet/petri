use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::types::PROTOCOL_VERSION;

#[derive(Debug, serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
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

/// The error envelope's machine-readable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub(crate) enum ErrorCode {
    InvalidRequest,
    InvalidStateTransition,
    ValidationRejected,
    NotFound,
    InternalError,
}

/// Context block of the normative error envelope
/// (`docs/reference/v3-server-api-protocol-spec.md` §6). Every field is
/// optional, and the whole block is omitted when no field applies.
#[derive(Debug, Default, serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct ErrorDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    endpoint: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    field_errors: Option<Vec<FieldError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    expected_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    current_state: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct ErrorPayload {
    code: ErrorCode,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    details: Option<ErrorDetails>,
}

/// The body of every error response.
#[derive(Debug, serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub(crate) struct ErrorEnvelope {
    protocol_version: &'static str,
    error: ErrorPayload,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match self {
            AppError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidRequest,
                msg,
                None,
            ),
            AppError::InvalidStateTransition { expected, current } => (
                StatusCode::CONFLICT,
                ErrorCode::InvalidStateTransition,
                format!("current state is '{current}'"),
                Some(ErrorDetails {
                    expected_state: expected,
                    current_state: Some(current),
                    ..ErrorDetails::default()
                }),
            ),
            AppError::ValidationRejected {
                field_errors,
                endpoint,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorCode::ValidationRejected,
                format!("validation failed for {endpoint}"),
                Some(ErrorDetails {
                    endpoint: Some(endpoint),
                    field_errors: (!field_errors.is_empty()).then_some(field_errors),
                    ..ErrorDetails::default()
                }),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, ErrorCode::NotFound, msg, None),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                msg,
                None,
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
