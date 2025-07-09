use crate::{
    error::ApiError, models::note::Note, repository::todo_repo::TodoRepo,
    routes::notes::AllNotesResponse,
};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{bson::doc, bson::oid::ObjectId, options::*, Collection};
use tracing::error;

#[async_trait]
pub trait NoteRepo: Send + Sync {
    async fn create_note(
        &self,
        user_id: ObjectId,
        title: &str,
        content: &str,
        tags: Vec<String>,
    ) -> Result<Note, ApiError>;
    async fn delete_note(&self, note_id: ObjectId, user_id: ObjectId) -> Result<(), ApiError>;
    async fn update_note(
        &self,
        user_id: ObjectId,
        note_id: ObjectId,
        title: &str,
        content: &str,
        tags: Vec<String>,
    ) -> Result<(), ApiError>;
    async fn get_note_by_id(&self, note_id: ObjectId, user_id: ObjectId) -> Result<Note, ApiError>;
    async fn get_all_notes_from_user(
        &self,
        user_id: ObjectId,
    ) -> Result<Vec<AllNotesResponse>, ApiError>;
    async fn pin_todo_list<T: TodoRepo>(
        &self,
        todo_repo: &T,
        todo_list_id: ObjectId,
        note_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<(), ApiError>;
    async fn unpin_todo_list(
        &self,
        todo_list_id: ObjectId,
        note_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<(), ApiError>;
}
pub struct MongoNoteRepo {
    collection: Collection<Note>,
}

impl MongoNoteRepo {
    pub fn new(collection: Collection<Note>) -> Self {
        Self { collection }
    }
}
#[async_trait]
impl NoteRepo for MongoNoteRepo {
    async fn create_note(
        &self,
        user_id: ObjectId,
        title: &str,
        content: &str,
        tags: Vec<String>,
    ) -> Result<Note, ApiError> {
        let new_note = Note {
            id: ObjectId::new(),
            user_id,
            title: title.to_string(),
            content: content.to_string(),
            tags,
            todo_lists: vec![],
        };
        match self.collection.insert_one(&new_note).await {
            Ok(res) => {
                if let Some(_object_id) = res.inserted_id.as_object_id() {
                    Ok(new_note)
                } else {
                    Err(ApiError::NothingChanged)
                }
            }
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }

    async fn delete_note(&self, note_id: ObjectId, user_id: ObjectId) -> Result<(), ApiError> {
        let filter = doc! {
            "_id": note_id,
            "client_id": user_id
        };
        match self.collection.find_one_and_delete(filter).await {
            Ok(result) => {
                if let Some(_result) = result {
                    //println!("{:?}", result);
                    return Ok(());
                }
                Err(ApiError::NotFound)
            }
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }

    async fn update_note(
        &self,
        user_id: ObjectId,
        note_id: ObjectId,
        title: &str,
        content: &str,
        tags: Vec<String>,
    ) -> Result<(), ApiError> {
        let filter = doc! {"_id": note_id, "client_id": user_id, };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
        match self
            .collection
            .find_one_and_update(
                filter,
                doc! {"$set": {
                        "title": title,
                        "content": content,
                        "tags": tags
                    }
                },
            )
            .with_options(options)
            .await
        {
            Ok(Some(_data)) => Ok(()),
            Ok(None) => Err(ApiError::NothingChanged),
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }

    async fn get_note_by_id(&self, note_id: ObjectId, user_id: ObjectId) -> Result<Note, ApiError> {
        let filter = doc! {"user_id": user_id, "_id": note_id};
        match self.collection.find_one(filter).await {
            Ok(Some(note)) => Ok(note),
            Ok(None) => Err(ApiError::NotFound),
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }
    async fn get_all_notes_from_user(
        &self,
        user_id: ObjectId,
    ) -> Result<Vec<AllNotesResponse>, ApiError> {
        let filter = doc! {"user_id": user_id};
        match self.collection.find(filter).await {
            Ok(cursor) => {
                let notes: Vec<AllNotesResponse> = cursor
                    .filter_map(|doc| async {
                        match doc {
                            Ok(info) => Some(AllNotesResponse {
                                title: info.title,
                                id: info.id,
                                tags: info.tags,
                            }),
                            //?
                            Err(_err) => None,
                        }
                    })
                    .collect()
                    .await;
                Ok(notes)
            }
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }
    async fn pin_todo_list<T: TodoRepo>(
        &self,
        todo_repo: &T,
        todo_list_id: ObjectId,
        note_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<(), ApiError> {
        //check if todo_list exist
        let todo_list = todo_repo.get_todo_list(todo_list_id, user_id).await?;

        //add todo_list to todo_list vec
        self.collection
            .update_one(
                doc! {"_id": note_id, "user_id": user_id},
                doc! { "$push" : {
                    "todo_lists": todo_list.id
                }},
            )
            .await
            .map_err(|err| {
                error!("{}", err);
                ApiError::InternalError
            })?;

        Ok(())
    }
    async fn unpin_todo_list(
        &self,
        todo_list_id: ObjectId,
        note_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<(), ApiError> {
        //check if todo_list is in todo_list vec
        //delete todo_list from todo_list vec
        match self
            .collection
            .find_one_and_update(
                doc! {"_id": note_id,"user_id": user_id, "todo_lists": todo_list_id},
                doc! {"$pull": {"todo_lists": todo_list_id}},
            )
            .await
        {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(ApiError::NotFound),
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }
}
