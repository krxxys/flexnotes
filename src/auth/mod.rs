use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::KEYS;

#[warn(dead_code)]
pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, AuthError> {
    let headers = req.headers();
    let token = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(AuthError::MissingCredentials)?;

    let token = token.trim_start_matches("Bearer ");
    let key = &KEYS.decoding;
    let validation = Validation::new(Algorithm::ES256);

    match decode::<Claims>(token, key, &validation) {
        Ok(token_data) => {
            println!("Authenicated user: {}", token_data.claims.email);

            let mut req = req;
            req.extensions_mut().insert(Arc::new(token_data));

            Ok(next.run(req).await)
        }
        Err(_) => Err(AuthError::InvalidToken),
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &KEYS.decoding,
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AuthError::InvalidToken)?;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub company: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseBody {
    pub acces_token: String,
    pub token_type: String,
    pub username: String,
}

impl AuthResponseBody {
    pub fn new(acces_token: String, username: String) -> Self {
        Self {
            acces_token,
            token_type: "Bearer".to_string(),
            username,
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    UserAlreadyExists,
    InternalServerError
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, erorr_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid Token"),
            AuthError::UserAlreadyExists => (StatusCode::FOUND, "User already exists"),
            AuthError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        };
        let body = Json(json!({
            "error": erorr_message
        }));

        (status, body).into_response()
    }
}
