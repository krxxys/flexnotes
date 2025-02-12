use std::{
    cell::Ref,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{extract::State, Json};
use bcrypt::{verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use mongodb::bson::{doc, oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
use sled::Db;

use crate::{
    auth::{generate_acces_token, generate_refresh_token, AuthError, AuthResponseBody, Claims},
    models::{self, DB},
    UserInfo, KEYS,
};

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

pub async fn authorize(
    State(db): State<DB>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponseBody>, AuthError> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    let user = db.get_user(&payload.username).await?;

    match verify(payload.password, &user.password) {
        Ok(true) => {
            let token = generate_acces_token(&user.email)?;
            let refresh_token = generate_refresh_token(&user.email)?;
            Ok(Json(AuthResponseBody::new(
                token,
                refresh_token,
                user.username,
            )))
        }
        Ok(false) => Err(AuthError::WrongCredentials),
        Err(_) => Err(AuthError::InternalServerError),
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
) -> Result<Json<AuthResponseBody>, AuthError> {
    if payload.username.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    if !db.user_exist(&payload.username, &payload.email).await? {
        let user = db
            .create_user(&payload.username, &payload.email, &payload.password)
            .await?;
    }

    let token = generate_acces_token(&payload.email)?;
    let refresh_token = generate_refresh_token(&payload.email)?;
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
) -> Result<Json<RefreshResponse>, AuthError> {
    let refresh_token = payload.refresh_token;
    match decode::<Claims>(
        &refresh_token,
        &KEYS.decoding,
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => {
            if token_data.claims.exp < DateTime::now().timestamp_millis() as usize {
                return Err(AuthError::InvalidToken);
            }

            let new_acces_token = generate_acces_token(&token_data.claims.email)?;
            let new_refresh_token = generate_refresh_token(&token_data.claims.email)?;

            Ok(Json(RefreshResponse {
                acces_token: new_acces_token,
                refresh_token: new_refresh_token,
            }))
        }
        Err(_) => Err(AuthError::InvalidToken),
    }
}
