use crate::{
    error::ApiError,
    models::note::Note,
    repository::{note_repo::NoteRepo, todo_repo::TodoRepo},
    routes::notes::AllNotesResponse,
};
use mongodb::bson::oid::ObjectId;

pub async fn create_note<R: NoteRepo>(
    repo: &R,
    user_id: ObjectId,
    title: &str,
    content: &str,
    tags: Vec<String>,
) -> Result<Note, ApiError> {
    let create_res = repo.create_note(user_id, title, content, tags).await?;
    Ok(create_res)
}

pub async fn delete_note<R: NoteRepo>(
    repo: &R,
    note_id: ObjectId,
    user_id: ObjectId,
) -> Result<(), ApiError> {
    repo.delete_note(note_id, user_id).await?;
    Ok(())
}

pub async fn update_note<R: NoteRepo>(
    repo: &R,
    user_id: ObjectId,
    note_id: ObjectId,
    title: &str,
    content: &str,
    tags: Vec<String>,
) -> Result<(), ApiError> {
    repo.update_note(user_id, note_id, title, content, tags)
        .await?;
    Ok(())
}

pub async fn get_note_by_id<R: NoteRepo>(
    repo: &R,
    user_id: ObjectId,
    note_id: ObjectId,
) -> Result<Note, ApiError> {
    let res = repo.get_note_by_id(note_id, user_id).await?;
    Ok(res)
}

pub async fn get_all_notes_from_user<R: NoteRepo>(
    repo: &R,
    user_id: ObjectId,
) -> Result<Vec<AllNotesResponse>, ApiError> {
    let res = repo.get_all_notes_from_user(user_id).await?;
    Ok(res)
}

pub async fn pin_todo_list<R: NoteRepo, T: TodoRepo>(
    note_repo: &R,
    todo_repo: &T,
    user_id: ObjectId,
    note_id: ObjectId,
    todo_list_id: ObjectId,
) -> Result<(), ApiError> {
    note_repo
        .pin_todo_list(todo_repo, todo_list_id, note_id, user_id)
        .await?;
    Ok(())
}

pub async fn unpin_todo_list<R: NoteRepo>(
    note_repo: &R,
    user_id: ObjectId,
    note_id: ObjectId,
    todo_list_id: ObjectId,
) -> Result<(), ApiError> {
    note_repo
        .unpin_todo_list(todo_list_id, note_id, user_id)
        .await?;
    Ok(())
}
