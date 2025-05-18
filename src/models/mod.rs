use std::sync::Arc;

use axum::http::StatusCode;
use bcrypt::DEFAULT_COST;
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, to_bson},
    options::{FindOneAndUpdateOptions, ReturnDocument},
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    routes::notes::{AllNotesReponse, CreateNotePayload},
};

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
    pub todo_list: Vec<TodoInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoInfo {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub title: String,
    pub status: bool,
    pub priority: TodoPriority,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum TodoPriority {
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone)]
pub struct DatabaseModel {
    pub notes: Collection<NoteInfo>,
    pub users: Collection<UserInfo>,
}

pub type DB = Arc<DatabaseModel>;

impl DatabaseModel {
    pub async fn get_user(&self, username: &str) -> Result<UserInfo, AppError> {
        let filter = doc! {"username": username};
        match self
            .users
            .find_one(filter)
            .await
            .map_err(|_| AppError::unauthorized())?
        {
            Some(user) => Ok(user),
            None => Err(AppError::unauthorized()),
        }
    }

    pub async fn user_exist(&self, username: &str, email: &str) -> Result<bool, AppError> {
        match self
            .users
            .find_one(doc! {"$or": [{"username": username}, {"email": email}]})
            .await
        {
            Ok(Some(_)) => Err(AppError::user_exist()),
            Ok(None) => Ok(false),
            Err(_) => Err(AppError::internal_error()),
        }
    }

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<UserInfo, AppError> {
        let new_user = UserInfo {
            id: ObjectId::new(),
            username: username.to_string(),
            email: email.to_string(),
            password: bcrypt::hash(password, DEFAULT_COST)
                .map_err(|_| AppError::internal_error())?,
        };

        match self.users.insert_one(&new_user).await {
            Ok(_res) => {
                println!("User created");
                Ok(new_user)
            }
            Err(err) => {
                println!("Error: {:?}", err);
                Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "User cannot be created :(",
                ))
            }
        }
    }

    pub async fn create_note(
        &self,
        data: CreateNotePayload,
        user: UserInfo,
    ) -> Result<ObjectId, AppError> {
        let new_note = NoteInfo {
            id: ObjectId::new(),
            client_id: user.id,
            title: data.title,
            content: data.content,
            tags: data.tags,
            todo_list: vec![],
        };
        match self.notes.insert_one(new_note).await {
            Ok(res) => {
                if let Some(object_id) = res.inserted_id.as_object_id() {
                    Ok(object_id)
                } else {
                    Err(AppError::internal_error())
                }
            }
            Err(_) => Err(AppError::internal_error()),
        }
    }

    pub async fn delete_note(&self, id: ObjectId, user: UserInfo) -> Result<StatusCode, AppError> {
        let filter = doc! {
            "_id": id,
            "client_id": user.id
        };
        match self.notes.find_one_and_delete(filter).await {
            Ok(result) => {
                if let Some(result) = result {
                    println!("{:?}", result);
                    return Ok(StatusCode::OK);
                }
                Err(AppError::not_found())
            }
            Err(_) => Err(AppError::internal_error()),
        }
    }

    pub async fn update_note(
        &self,
        data: CreateNotePayload,
        user: UserInfo,
        id: ObjectId,
    ) -> Result<NoteInfo, AppError> {
        let filter = doc! {"_id": id, "client_id": user.id, };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
        match self
            .notes
            .find_one_and_update(
                filter,
                doc! {"$set": {
                        "title": data.title,
                        "content": data.content,
                        "tags": data.tags
                    }
                },
            )
            .with_options(options)
            .await
        {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(AppError::new(StatusCode::BAD_REQUEST, "Note not updated")),
            Err(_err) => Err(AppError::internal_error()),
        }
    }

    pub async fn note_by_id(&self, id: ObjectId, user: UserInfo) -> Result<NoteInfo, AppError> {
        let filter = doc! {"client_id": user.id, "_id": id};
        match self.notes.find_one(filter).await {
            Ok(Some(note)) => Ok(note),
            Ok(None) => Err(AppError::not_found()),
            Err(_) => Err(AppError::internal_error()),
        }
    }

    pub async fn all_notes_from_user(
        &self,
        user: UserInfo,
    ) -> Result<Vec<AllNotesReponse>, AppError> {
        let filter = doc! {"client_id": user.id};
        match self.notes.find(filter).await {
            Ok(cursor) => {
                let notes: Vec<AllNotesReponse> = cursor
                    .filter_map(|doc| async {
                        match doc {
                            Ok(info) => Some(AllNotesReponse {
                                title: info.title,
                                id: info.id,
                                tags: info.tags,
                            }),
                            Err(_err) => None,
                        }
                    })
                    .collect()
                    .await;
                Ok(notes)
            }
            Err(err) => {
                eprintln!("{:?}", err);
                Err(AppError::internal_error())
            }
        }
    }
    pub async fn create_todo(
        &self,
        user: UserInfo,
        note_id: ObjectId,
        todo_title: String,
        todo_status: bool,
        todo_priority: TodoPriority,
    ) -> Result<StatusCode, AppError> {
        let filter = doc! {"client_id": user.id, "_id": note_id};
        let new_todo = TodoInfo {
            id: ObjectId::new(),
            title: todo_title,
            status: todo_status,
            priority: todo_priority,
        };
        match to_bson(&new_todo) {
            Ok(todo_bson) => {
                match self
                    .notes
                    .update_one(filter, doc! {"$push" : { "todo_list": todo_bson }})
                    .await
                {
                    Ok(res) => {
                        if res.modified_count > 0 {
                            Ok(StatusCode::OK)
                        } else {
                            Err(AppError::not_found())
                        }
                    }
                    Err(_err) => Err(AppError::internal_error()),
                }
            }
            Err(err) => {
                eprintln!("Error ocured: {:?}", err);
                Err(AppError::internal_error())
            }
        }
    }
    pub async fn update_todo(
        &self,
        user: UserInfo,
        note_id: ObjectId,
        todo_id: ObjectId,
        title: String,
        status: bool,
        priority: TodoPriority,
    ) -> Result<StatusCode, AppError> {
        let filter = doc! {
            "client_id": user.id,
             "_id": note_id,
             "todo_list._id": todo_id
        };

        let priority = match to_bson(&priority) {
            Ok(bson) => bson,
            Err(err) => {
                eprintln!("Error occured: {:?}", err);
                return Err(AppError::internal_error());
            }
        };

        let update = doc! {"$set": {
            "todo_list.$.title": title,
            "todo_list.$.status": status,
            "todo_list.$.priority": priority
        }};

        match self.notes.update_one(filter, update).await {
            Ok(res) => {
                if res.matched_count > 0 {
                    Ok(StatusCode::OK)
                } else {
                    Ok(StatusCode::NOT_MODIFIED)
                }
            }
            Err(err) => {
                eprintln!("Error occured: {:?}", err);
                Err(AppError::internal_error())
            }
        }
    }
    pub async fn delete_todo(
        &self,
        user: UserInfo,
        note_id: ObjectId,
        todo_id: ObjectId,
    ) -> Result<StatusCode, AppError> {
        let filter = doc! { "_id": note_id, "client_id": user.id, "todo_list._id": todo_id};
        let delete = doc! {"$pull": { //pull removes element
            "todo_list": {
                "_id": todo_id
            }
        }};

        match self.notes.update_one(filter, delete).await {
            Ok(res) => {
                if res.modified_count > 0 {
                    Ok(StatusCode::OK)
                } else {
                    Ok(StatusCode::NOT_MODIFIED)
                }
            }
            Err(err) => {
                eprintln!("Error occured: {:?}", err);
                Err(AppError::internal_error())
            }
        }
    }
    pub async fn get_todos_by_note_id(
        &self,
        user: UserInfo,
        note_id: ObjectId,
    ) -> Result<Vec<TodoInfo>, AppError> {
        let filter = doc! {"_id": note_id, "client_id": user.id};
        match self.notes.find_one(filter).await {
            Ok(Some(note)) => Ok(note.todo_list),
            Ok(None) => Ok(vec![]),
            Err(err) => {
                eprintln!("Error occured: {:?}", err);
                Err(AppError::internal_error())
            }
        }
    }
}
