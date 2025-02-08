use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    AuthorNotFound
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

impl DatabaseModel {
    pub async fn get_user(&self, email: String) -> Option<UserInfo> {
        self.users
            .find_one(doc! {"email": email})
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            .unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseModel {
    pub notes: Collection<NoteInfo>,
    pub users: Collection<UserInfo>,
}

pub type DB = Arc<DatabaseModel>;
