use mongodb::bson::oid::ObjectId;

use crate::{
    error::ApiError,
    models::todo::{TodoList, TodoPriority},
    repository::todo_repo::TodoRepo,
};

pub async fn create_todo_list<R: TodoRepo>(
    repo: &R,
    user_id: ObjectId,
    title: String,
) -> Result<TodoList, ApiError> {
    match repo.create_todo_list(title, user_id).await {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

pub async fn get_all_todo_list<R: TodoRepo>(
    repo: &R,
    user_id: ObjectId,
) -> Result<Vec<TodoList>, ApiError> {
    repo.get_all_todo_lists(user_id).await
}

pub async fn get_todo_lists<R: TodoRepo>(
    repo: &R,
    list: Vec<ObjectId>,
    user_id: ObjectId,
) -> Result<Vec<TodoList>, ApiError> {
    repo.get_todo_lists(list, user_id).await
}

pub async fn delete_todo_list<R: TodoRepo>(
    repo: &R,
    todo_list_id: ObjectId,
    user_id: ObjectId,
) -> Result<(), ApiError> {
    repo.delete_todo_list(todo_list_id, user_id).await
}

pub async fn rename_todo_list<R: TodoRepo>(
    repo: &R,
    todo_list_id: ObjectId,
    user_id: ObjectId,
    title: String,
) -> Result<(), ApiError> {
    repo.rename_todo_list(todo_list_id, user_id, title).await
}

pub async fn create_todo<R: TodoRepo>(
    repo: &R,
    todo_list_id: ObjectId,
    user_id: ObjectId,
    title: String,
    status: bool,
    priority: TodoPriority,
) -> Result<(), ApiError> {
    repo.create_todo(todo_list_id, user_id, title, status, priority)
        .await
}

pub async fn modify_todo<R: TodoRepo>(
    repo: &R,
    todo_list_id: ObjectId,
    user_id: ObjectId,
    todo_id: ObjectId,
    title: String,
    status: bool,
    priority: TodoPriority,
) -> Result<(), ApiError> {
    repo.modify_todo(todo_list_id, user_id, todo_id, title, status, priority)
        .await
}

pub async fn delete_todo<R: TodoRepo>(
    repo: &R,
    todo_list_id: ObjectId,
    user_id: ObjectId,
    todo_id: ObjectId,
) -> Result<(), ApiError> {
    repo.delete_todo(todo_list_id, user_id, todo_id).await
}
