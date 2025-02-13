use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};

use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    error::AppError,
    models::{NoteInfo, DB},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNotePayload {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

pub async fn create_note(
    State(db): State<DB>,
    Extension(claims): Auth,
    Json(payload): Json<CreateNotePayload>,
) -> Result<Json<ObjectId>, AppError> {
    let user = db.get_user(&claims.username).await?;
    let note = db.create_note(payload, user).await?;
    Ok(Json(note))
}

pub async fn delete_note(
    State(db): State<DB>,
    Extension(claims): Auth,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let user = db.get_user(&claims.username).await?;
    return db.delete_note(id, user).await;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AllNotesReponse {
    pub title: String,
    pub id: ObjectId,
    pub tags: Vec<String>,
}

pub async fn get_all_notes_info(
    State(db): State<DB>,
    Extension(claims): Auth,
) -> Result<Json<Vec<AllNotesReponse>>, AppError> {
    let user = db.get_user(&claims.username).await?;
    let all_notes = db.all_notes_from_user(user).await?;
    Ok(Json(all_notes))
}

pub async fn get_note_by_id(
    State(db): State<DB>,
    Extension(claims): Auth,
    Path(id): Path<String>,
) -> Result<Json<NoteInfo>, AppError> {
    let user = db.get_user(&claims.username).await?;
    let note = db.note_by_id(id, user).await?;
    Ok(Json(note))
}

pub async fn update_note_by_id(
    State(db): State<DB>,
    Extension(claims): Auth,
    Path(id): Path<String>,
    Json(payload): Json<CreateNotePayload>,
) -> Result<Json<NoteInfo>, AppError> {
    let user = db.get_user(&claims.username).await?;
    let updated_note = db.update_note(payload, user, id).await?;
    Ok(Json(updated_note))
}
