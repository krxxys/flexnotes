use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
}

impl AppError {
    pub fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status_code,
            message: message.into(),
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Unauthorized".into(),
        }
    }

    pub fn internal_error() -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Internal server error".into(),
        }
    }

    pub fn user_exist() -> Self {
        Self {
            status_code: StatusCode::FOUND,
            message: "User with this username or email already exists".into(),
        }
    }

    pub fn token_expired() -> Self {
        Self {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Token expired".into(),
        }
    }

    pub fn missign_credentials() -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            message: "Missing credentials".into(),
        }
    }
    pub fn not_found() -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            message: "Not found".into(),
        }
    }

    pub fn bad_request() -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            message: "Something is missing".into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let mut res = (self.status_code, Json(json!({"error": self.message}))).into_response();
        res.extensions_mut().insert(self);
        res
    }
}
