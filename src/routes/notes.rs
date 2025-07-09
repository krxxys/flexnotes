use axum::{
    extract::{Path, State},
    Extension, Json,
};

use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{auth::AuthUser, error::ApiError, models::note::Note, services, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNotePayload {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

pub async fn create_note(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Json(payload): Json<CreateNotePayload>,
) -> Result<Json<Note>, ApiError> {
    let note = services::note_service::create_note(
        &app_state.database.note_repo(),
        user.id,
        payload.title.as_str(),
        payload.content.as_str(),
        payload.tags,
    )
    .await?;
    Ok(Json(note))
}

pub async fn delete_note(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path(id): Path<ObjectId>,
) -> Result<(), ApiError> {
    services::note_service::delete_note(&app_state.database.note_repo(), id, user.id).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AllNotesResponse {
    pub title: String,
    pub id: ObjectId,
    pub tags: Vec<String>,
}

pub async fn get_all_notes_info(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
) -> Result<Json<Vec<AllNotesResponse>>, ApiError> {
    let all_notes =
        services::note_service::get_all_notes_from_user(&app_state.database.note_repo(), user.id)
            .await?;
    Ok(Json(all_notes))
}

pub async fn get_note_by_id(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path(id): Path<ObjectId>,
) -> Result<Json<Note>, ApiError> {
    let note = services::note_service::get_note_by_id(&app_state.database.note_repo(), user.id, id)
        .await?;
    Ok(Json(note))
}

pub async fn update_note_by_id(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path(id): Path<ObjectId>,
    Json(payload): Json<CreateNotePayload>,
) -> Result<(), ApiError> {
    services::note_service::update_note(
        &app_state.database.note_repo(),
        user.id,
        id,
        &payload.title,
        &payload.content,
        payload.tags.to_owned(),
    )
    .await?;
    Ok(())
}

pub async fn pin_todo_list(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path((id, todo_list_id)): Path<(ObjectId, ObjectId)>,
) -> Result<(), ApiError> {
    services::note_service::pin_todo_list(
        &app_state.database.note_repo(),
        &app_state.database.todos_repo(),
        user.id,
        id,
        todo_list_id,
    )
    .await?;
    Ok(())
}

pub async fn unpin_todo_list(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path((id, todo_list_id)): Path<(ObjectId, ObjectId)>,
) -> Result<(), ApiError> {
    services::note_service::unpin_todo_list(
        &app_state.database.note_repo(),
        user.id,
        id,
        todo_list_id,
    )
    .await?;
    Ok(())
}
