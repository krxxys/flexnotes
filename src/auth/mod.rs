use crate::{error::AppError, KEYS};
use axum::{
    body::Body, extract::FromRequestParts, http::Request, middleware::Next, response::Response,
    Extension, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub type Auth = Extension<Arc<TokenData<Claims>>>;

pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let headers = req.headers();
    let token = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(AppError::unauthorized())?;

    let token = token.trim_start_matches("Bearer ");
    let key = &KEYS.decoding;
    let validation = Validation::new(Algorithm::HS256);

    match decode::<Claims>(token, key, &validation) {
        Ok(token_data) => {
            println!("Authenicated user: {}", token_data.claims.username);

            let mut req = req;
            req.extensions_mut().insert(Arc::new(token_data));

            Ok(next.run(req).await)
        }
        Err(err) => {
            println!("Error {}", err);
            Err(AppError::unauthorized())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub company: String,
    pub exp: usize,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::unauthorized())?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &KEYS.decoding,
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AppError::unauthorized())?;
        Ok(token_data.claims)
    }
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
    pub acces_token: String,
    pub token_type: String,
    pub refresh_token: String,
    pub username: String,
}

impl AuthResponseBody {
    pub fn new(acces_token: String, refresh_token: String, username: String) -> Self {
        Self {
            acces_token,
            token_type: "Bearer".to_string(),
            refresh_token,
            username,
        }
    }
}

pub fn generate_acces_token(username: &str) -> Result<String, AppError> {
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
        .map_err(|_err| AppError::internal_error())?;

    Ok(token)
}

pub fn generate_refresh_token(username: &str) -> Result<String, AppError> {
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
        .map_err(|_err| AppError::internal_error())?;

    Ok(token)
}
