use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::State, Json};
use bcrypt::{verify, DEFAULT_COST};
use jsonwebtoken::{encode, Algorithm, Header};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{AuthError, AuthResponseBody, Claims},
    models::DB,
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
    let filter = doc! {"username": payload.username};
    let result = db
        .users
        .find_one(filter)
        .await
        .map_err(|_| AuthError::WrongCredentials)?;

    println!("{:?}", result);
    match result {
        Some(data) => {
            match verify(payload.password, &data.password) {
                Ok(true) => {
                    let claims = Claims {
                        email: data.email,
                        company: "TEST".to_owned(),
                        exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize + 2 * 60 * 60,
                    };
    
                    let token = encode(&Header::new(Algorithm::HS256), &claims, &KEYS.encoding)
                        .map_err(|_err| return AuthError::TokenCreation)?;
    
                    return Ok(Json(AuthResponseBody::new(token, data.username)))
                },
                Ok(false) => {
                    return Err(AuthError::WrongCredentials)
                }
                Err(err) => {
                    println!("Error: {}", err);
                    return Err(AuthError::InternalServerError)
                },
            }
        }
        None => {
            return Err(AuthError::WrongCredentials)
        }, 
}}




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
    //check if user with this username or email exists
    if let Some(_user_exist) = db
        .users
        .find_one(doc! {"$or": [{"username": &payload.username}, {"email": &payload.email}]})
        .await
        .unwrap()
    {
        Err(AuthError::UserAlreadyExists)
    } else {
        let new_user = UserInfo {
            id: ObjectId::new(),
            username: payload.username.clone(),
            email: payload.email.clone(),
            password: bcrypt::hash(payload.password, DEFAULT_COST).map_err(|_| return AuthError::TokenCreation)?,
        };

        match db.users.insert_one(new_user).await {
            Ok(_result) => {
                let claims = Claims {
                    email: payload.email,
                    company: "flexnote".to_owned(),
                    exp: 2000000000,
                };

                let token = encode(&Header::new(Algorithm::HS256), &claims, &KEYS.encoding)
                    .map_err(|_err| AuthError::TokenCreation)?;

                Ok(Json(AuthResponseBody::new(token, payload.username)))
            }
            Err(insert_error) => {
                eprintln!("{:?}", insert_error);
                Err(AuthError::TokenCreation)
            }
        }
    }
}
