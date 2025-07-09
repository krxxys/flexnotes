use crate::{
    auth::AuthUser,
    error::ApiError,
    models::todo::{TodoList, TodoPriority},
    services::{self},
    AppState,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoListPayload {
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoListVecPayload {
    list: Vec<ObjectId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoPayload {
    pub title: String,
    pub status: bool,
    pub priority: TodoPriority,
}

pub async fn create_todo_list(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Json(payload): Json<TodoListPayload>,
) -> Result<Json<TodoList>, ApiError> {
    match services::todo_service::create_todo_list(
        &app_state.database.todos_repo(),
        user.id,
        payload.title,
    )
    .await
    {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(err),
    }
}

pub async fn get_all_todo_lists(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
) -> Result<Json<Vec<TodoList>>, ApiError> {
    match services::todo_service::get_all_todo_list(&app_state.database.todos_repo(), user.id).await
    {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(err),
    }
}

pub async fn get_all_todos_by_id(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Json(payload): Json<TodoListVecPayload>,
) -> Result<Json<Vec<TodoList>>, ApiError> {
    match services::todo_service::get_todo_lists(
        &app_state.database.todos_repo(),
        payload.list,
        user.id,
    )
    .await
    {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(err),
    }
}

pub async fn rename_todo_list(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path(todo_list_id): Path<ObjectId>,
    Json(payload): Json<TodoListPayload>,
) -> Result<(), ApiError> {
    match services::todo_service::rename_todo_list(
        &app_state.database.todos_repo(),
        todo_list_id,
        user.id,
        payload.title,
    )
    .await
    {
        Ok(_res) => Ok(()),
        Err(err) => Err(err),
    }
}

pub async fn delete_todo_list(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path(todo_list_id): Path<ObjectId>,
) -> Result<(), ApiError> {
    match services::todo_service::delete_todo_list(
        &app_state.database.todos_repo(),
        todo_list_id,
        user.id,
    )
    .await
    {
        Ok(_res) => Ok(()),
        Err(err) => Err(err),
    }
}

pub async fn create_todo(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path(todo_list_id): Path<ObjectId>,
    Json(payload): Json<TodoPayload>,
) -> Result<(), ApiError> {
    match services::todo_service::create_todo(
        &app_state.database.todos_repo(),
        todo_list_id,
        user.id,
        payload.title,
        payload.status,
        payload.priority,
    )
    .await
    {
        Ok(_res) => Ok(()),
        Err(err) => Err(err),
    }
}

pub async fn modify_todo(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path((todo_list_id, todo_id)): Path<(ObjectId, ObjectId)>,
    Json(payload): Json<TodoPayload>,
) -> Result<(), ApiError> {
    match services::todo_service::modify_todo(
        &app_state.database.todos_repo(),
        todo_list_id,
        user.id,
        todo_id,
        payload.title,
        payload.status,
        payload.priority,
    )
    .await
    {
        Ok(_res) => Ok(()),
        Err(err) => Err(err),
    }
}

pub async fn delete_todo(
    State(app_state): State<AppState>,
    Extension(user): AuthUser,
    Path((todo_list_id, todo_id)): Path<(ObjectId, ObjectId)>,
) -> Result<(), ApiError> {
    match services::todo_service::delete_todo(
        &app_state.database.todos_repo(),
        todo_list_id,
        user.id,
        todo_id,
    )
    .await
    {
        Ok(_res) => Ok(()),
        Err(err) => Err(err),
    }
}
