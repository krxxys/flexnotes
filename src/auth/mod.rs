use crate::error::ApiError;
use crate::repository::user_repo::UserRepo;
use crate::AppState;
use crate::{models::user::User, KEYS};
use axum::routing::Route;
use axum::{
    body::Body, extract::State, http::Request, middleware::Next, response::Response, Extension,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::error;

pub type AuthUser = Extension<Arc<User>>;

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let headers = req.headers();
    let token = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(ApiError::Unathorized)?;

    let token = token.trim_start_matches("Bearer ");
    let key = &KEYS.decoding;
    let validation = Validation::new(Algorithm::HS256);

    match decode::<Claims>(token, key, &validation) {
        Ok(token_data) => {
            println!("Authenicated user: {}", token_data.claims.username);

            let user = app_state
                .database
                .user_repo()
                .get_user(&token_data.claims.username)
                .await?;
            let mut req = req;
            req.extensions_mut().insert(Arc::new(token_data));
            req.extensions_mut().insert(Arc::new(user));

            Ok(next.run(req).await)
        }
        Err(err) => {
            println!("Error {}", err);
            Err(ApiError::InternalError)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub company: String,
    pub exp: usize,
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthResponseBody {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: String,
    pub username: String,
}

impl AuthResponseBody {
    pub fn new(access_token: String, refresh_token: String, username: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
            refresh_token,
            username,
        }
    }
}

pub fn generate_acces_token(username: &str) -> Result<String, ApiError> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 2 * 60 * 60; //2h

    let claims = Claims {
        username: username.to_owned(),
        company: "flexnotes".to_owned(),
        exp: expiration,
    };

    let token = jsonwebtoken::encode(&Header::new(Algorithm::HS256), &claims, &KEYS.encoding)
        .map_err(|err| {
            error!("{}", err.to_string());
            ApiError::InternalError
        })?;

    Ok(token)
}

pub fn generate_refresh_token(username: &str) -> Result<String, ApiError> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 2 * 24 * 60 * 60; //2days

    let claims = Claims {
        username: username.to_owned(),
        company: "flexnotes".to_owned(),
        exp: expiration,
    };

    let token = jsonwebtoken::encode(&Header::new(Algorithm::HS256), &claims, &KEYS.encoding)
        .map_err(|err| {
            error!("{}", err.to_string());
            ApiError::InternalError
        })?;

    Ok(token)
}
