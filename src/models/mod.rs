use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse, Json};
use bcrypt::DEFAULT_COST;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing_subscriber::registry::Data;

use crate::auth::{Auth, AuthError};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NoteInfo {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub client_id: ObjectId,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NotesError {
    DbError,
    NoteNotFound,
    AuthorNotFound,
}

impl IntoResponse for NotesError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            NotesError::DbError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create new note",
            ),
            NotesError::NoteNotFound => (StatusCode::NOT_FOUND, "Note not found"),
            NotesError::AuthorNotFound => (StatusCode::NOT_FOUND, "Author not found"),
        };
        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseModel {
    pub notes: Collection<NoteInfo>,
    pub users: Collection<UserInfo>,
}

pub type DB = Arc<DatabaseModel>;

impl DatabaseModel {
    pub async fn get_user(&self, username: &str) -> Result<UserInfo, AuthError> {
        let filter = doc! {"username": username};
        match self
            .users
            .find_one(filter)
            .await
            .map_err(|_| AuthError::WrongCredentials)?
        {
            Some(user) => Ok(user),
            None => Err(AuthError::WrongCredentials),
        }
    }

    pub async fn user_exist(&self, username: &str, email: &str) -> Result<bool, AuthError> {
        match self
            .users
            .find_one(doc! {"$or": [{"username": username}, {"email": email}]})
            .await
        {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(_) => Err(AuthError::InternalServerError),
        }
    }

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<UserInfo, AuthError> {
        let new_user = UserInfo {
            id: ObjectId::new(),
            username: username.to_string(),
            email: email.to_string(),
            password: bcrypt::hash(password, DEFAULT_COST)
                .map_err(|_| AuthError::InternalServerError)?,
        };

        Ok(new_user)
    }
}
