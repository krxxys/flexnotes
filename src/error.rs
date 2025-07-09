use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, Serialize)]
pub enum ApiError {
    #[error("Resource not found")]
    NotFound,
    #[error("Something went wrong")]
    InternalError,
    #[error("Something is missing")]
    MissingPayload,
    #[error("User unathorized")]
    Unathorized,
    #[error("User already exists")]
    UserExist,
    #[error("Your token expired")]
    TokenExpired,
    #[error("Missing auth.. ")]
    MissingCredential,
    #[error("Nothing changed")]
    NothingChanged,
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::MissingPayload => StatusCode::BAD_REQUEST,
            ApiError::Unathorized => StatusCode::UNAUTHORIZED,
            ApiError::UserExist => StatusCode::FOUND,
            ApiError::TokenExpired => StatusCode::UNAUTHORIZED,
            ApiError::MissingCredential => StatusCode::UNAUTHORIZED,
            ApiError::NothingChanged => StatusCode::NOT_MODIFIED,
        };

        let mut res = (status_code, self.to_string()).into_response();

        res.extensions_mut().insert(self);
        res
    }
}
