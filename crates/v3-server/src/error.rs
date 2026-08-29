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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message, extra) = match self {
            AppError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "invalid_request",
                msg,
                serde_json::Value::Null,
            ),
            AppError::InvalidStateTransition { expected, current } => (
                StatusCode::CONFLICT,
                "invalid_state_transition",
                format!("current state is '{current}'"),
                serde_json::json!({
                    "expected": expected,
                    "current": current,
                }),
            ),
            AppError::ValidationRejected {
                field_errors,
                endpoint,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_rejected",
                format!("validation failed for {endpoint}"),
                serde_json::json!({ "field_errors": field_errors }),
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "not_found",
                msg,
                serde_json::Value::Null,
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg,
                serde_json::Value::Null,
            ),
        };

        let mut body = serde_json::json!({
            "protocol_version": PROTOCOL_VERSION,
            "error": error_code,
            "message": message,
        });

        if !extra.is_null() {
            if let serde_json::Value::Object(ref mut map) = body {
                if let serde_json::Value::Object(extra_map) = extra {
                    map.extend(extra_map);
                }
            }
        }

        (status, axum::Json(body)).into_response()
    }
}
