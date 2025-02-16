use axum::{extract::State, Json};
use bcrypt::verify;
use jsonwebtoken::{decode, Algorithm, Validation};
use mongodb::bson::{doc, DateTime};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{generate_acces_token, generate_refresh_token, AuthResponseBody, Claims},
    error::AppError,
    models::DB,
    KEYS,
};

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

pub async fn authorize(
    State(db): State<DB>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponseBody>, AppError> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(AppError::missign_credentials());
    }

    let user = db.get_user(&payload.username).await?;

    match verify(payload.password, &user.password) {
        Ok(true) => {
            let token = generate_acces_token(&user.username)?;
            let refresh_token = generate_refresh_token(&user.username)?;
            Ok(Json(AuthResponseBody::new(
                token,
                refresh_token,
                user.username,
            )))
        }
        Ok(false) => Err(AppError::unauthorized()),
        Err(_) => Err(AppError::internal_error()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(db): State<DB>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<AuthResponseBody>, AppError> {
    if payload.username.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::missign_credentials());
    }

    if !db.user_exist(&payload.username, &payload.email).await? {
        let _user = db
            .create_user(&payload.username, &payload.email, &payload.password)
            .await?;
    }

    let token = generate_acces_token(&payload.username)?;
    let refresh_token = generate_refresh_token(&payload.username)?;
    Ok(Json(AuthResponseBody::new(
        token,
        refresh_token,
        payload.username,
    )))
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
) -> Result<Json<RefreshResponse>, AppError> {
    let refresh_token = payload.refresh_token;
    match decode::<Claims>(
        &refresh_token,
        &KEYS.decoding,
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => {
            if token_data.claims.exp < DateTime::now().timestamp_millis() as usize {
                return Err(AppError::token_expired());
            }

            let new_acces_token = generate_acces_token(&token_data.claims.username)?;
            let new_refresh_token = generate_refresh_token(&token_data.claims.username)?;

            Ok(Json(RefreshResponse {
                acces_token: new_acces_token,
                refresh_token: new_refresh_token,
            }))
        }
        Err(_) => Err(AppError::unauthorized()),
    }
}
