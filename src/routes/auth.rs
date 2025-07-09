use crate::{
    auth::{generate_acces_token, generate_refresh_token, AuthResponseBody, AuthUser, Claims},
    error::ApiError,
    services::user_service::{login_user, register_user},
    AppState, KEYS,
};
use axum::{extract::State, http::StatusCode, Extension, Json};
use jsonwebtoken::{decode, Algorithm, Validation};
use mongodb::bson::{doc, DateTime};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

pub async fn authorize(
    State(app_state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponseBody>, ApiError> {
    //println!("{:?}", payload);
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(ApiError::MissingCredential);
    }
    let response = login_user(
        &app_state.database.user_repo(),
        &payload.username,
        &payload.password,
    )
    .await?;
    Ok(Json(response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(app_state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<AuthResponseBody>, ApiError> {
    if payload.username.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        return Err(ApiError::MissingCredential);
    }
    let reponse = register_user(
        &app_state.database.user_repo(),
        &payload.username,
        &payload.email,
        &payload.password,
    )
    .await?;
    Ok(Json(reponse))
}

#[derive(Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshResponse {
    pub acces_token: String,
    pub refresh_token: String,
}

pub async fn refresh_token(
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, ApiError> {
    let refresh_token = payload.refresh_token;
    match decode::<Claims>(
        &refresh_token,
        &KEYS.decoding,
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => {
            if token_data.claims.exp < DateTime::now().timestamp_millis() as usize {
                return Err(ApiError::TokenExpired);
            }

            let new_acces_token = generate_acces_token(&token_data.claims.username)?;
            let new_refresh_token = generate_refresh_token(&token_data.claims.username)?;

            Ok(Json(RefreshResponse {
                acces_token: new_acces_token,
                refresh_token: new_refresh_token,
            }))
        }
        Err(err) => {
            error!("{}", err.to_string());
            Err(ApiError::InternalError)
        }
    }
}

pub async fn check_auth(Extension(_user): AuthUser) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::ACCEPTED)
}
