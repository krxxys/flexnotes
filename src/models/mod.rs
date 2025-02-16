use std::{str::FromStr, sync::Arc};

use axum::http::StatusCode;
use bcrypt::DEFAULT_COST;
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
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

        Ok(new_user)
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

    pub async fn delete_note(&self, id: String, user: UserInfo) -> Result<StatusCode, AppError> {
        let object_id = ObjectId::from_str(&id).unwrap();
        let filter = doc! {
            "_id": object_id,
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
        id: String,
    ) -> Result<NoteInfo, AppError> {
        let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "client_id": user.id, };
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

    pub async fn note_by_id(&self, id: String, user: UserInfo) -> Result<NoteInfo, AppError> {
        let filter = doc! {"client_id": user.id, "_id": ObjectId::from_str(&id).unwrap()};
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
}
