use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use crate::models::{ErrorResponse, ErrorDetail};

// Define our own error type
#[derive(Debug)]
pub enum AppError {
    ModelError(anyhow::Error),
    InvalidInput(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_detail) = match self {
            AppError::ModelError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetail {
                    message: format!("Model inference failed: {}", e),
                    error_type: "server_error".to_string(),
                    code: None,
                },
            ),
            AppError::InvalidInput(message) => (
                StatusCode::BAD_REQUEST,
                ErrorDetail {
                    message,
                    error_type: "invalid_request_error".to_string(),
                    code: None,
                },
            ),
            AppError::InternalServerError(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetail {
                    message,
                    error_type: "server_error".to_string(),
                    code: None,
                },
            ),
        };

        let body = Json(ErrorResponse { error: error_detail });
        (status, body).into_response()
    }
}

// This allows us to use `?` to convert from anyhow::Error
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::ModelError(err.into())
    }
}