use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    auth::AuthUser,
    error::AppError,
    models::{TodoInfo, TodoPriority, DB},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoPayload {
    title: String,
    status: bool,
    priority: TodoPriority,
}

pub async fn get_todos_by_note_id(
    State(db): State<DB>,
    Extension(user): AuthUser,
    Path(id): Path<ObjectId>,
) -> Result<Json<Vec<TodoInfo>>, AppError> {
    let todos = db.get_todos_by_note_id(user, id).await?;
    Ok(Json(todos))
}

pub async fn create_todo(
    State(db): State<DB>,
    Extension(user): AuthUser,
    Path(id): Path<ObjectId>,
    Json(payload): Json<TodoPayload>,
) -> Result<StatusCode, AppError> {
    match db
        .create_todo(user, id, payload.title, payload.status, payload.priority)
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

pub async fn update_todo_by_id(
    State(db): State<DB>,
    Extension(user): AuthUser,
    Path((id, todo_id)): Path<(ObjectId, ObjectId)>,
    Json(payload): Json<TodoPayload>,
) -> Result<StatusCode, AppError> {
    match db
        .update_todo(
            user,
            id,
            todo_id,
            payload.title,
            payload.status,
            payload.priority,
        )
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

pub async fn delete_todo_by_id(
    State(db): State<DB>,
    Extension(user): AuthUser,
    Path((id, todo_id)): Path<(ObjectId, ObjectId)>,
) -> Result<StatusCode, AppError> {
    match db.delete_todo(user, id, todo_id).await {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}
