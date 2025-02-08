use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};

use crate::{
    auth::Claims,
    models::{NoteInfo, NotesError, DB},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNotePayload {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

pub async fn create_note(
    State(db): State<DB>,
    claims: Claims,
    Json(payload): Json<CreateNotePayload>,
) -> Result<StatusCode, NotesError> {
    if let Some(user) = db
        .users
        .find_one(doc! {"email": claims.email.clone()})
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        .unwrap()
    {
        let new_note = NoteInfo {
            id: ObjectId::new(),
            client_id: user.id,
            title: payload.title,
            content: payload.content,
            tags: vec![],
        };
        match db.notes.insert_one(new_note).await {
            Ok(_res) => Ok(StatusCode::OK),
            Err(err) => {
                eprintln!("{:?}", err);
                Err(NotesError::DbError)
            }
        }
    } else {
        Err(NotesError::DbError)
    }
}

pub async fn delete_note(
    State(db): State<DB>,
    claims: Claims,
    Path(id): Path<String>,
) -> Result<StatusCode, NotesError> {
    if let Some(user) = db.get_user(claims.email).await {
        let object_id = ObjectId::from_str(&id).unwrap();
        let filter = doc! {
            "_id": object_id,
            "client_id": user.id
        };
        match db.notes.find_one_and_delete(filter).await {
            Ok(result) => {
                if let Some(result) = result {
                    println!("{:?}", result);
                    return Ok(StatusCode::OK);
                }
                Err(NotesError::NoteNotFound)
            }
            Err(_) => Err(NotesError::DbError),
        }
    } else {
        Err(NotesError::AuthorNotFound)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AllNotesReponse {
    title: String,
    id: ObjectId,
    tags: Vec<String>,
}

pub async fn get_all_notes_info(
    State(db): State<DB>,
    claims: Claims,
) -> Result<Json<Vec<AllNotesReponse>>, NotesError> {
    if let Some(user) = db.get_user(claims.email).await {
        let filter = doc! {"client_id": user.id};
        match db.notes.find(filter).await {
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
                Ok(Json(notes))
            }
            Err(err) => {
                eprintln!("{:?}", err);
                Err(NotesError::DbError)
            }
        }
    } else {
        Err(NotesError::AuthorNotFound)
    }
}

pub async fn get_note_by_id(
    State(db): State<DB>,
    claims: Claims,
    Path(id): Path<String>,
) -> Result<Json<NoteInfo>, NotesError> {
    if let Some(user) = db.get_user(claims.email).await {
        let filter = doc! {"client_id": user.id, "_id": ObjectId::from_str(&id).unwrap()};
        match db.notes.find_one(filter).await {
            Ok(Some(note)) => Ok(Json(note)),
            Ok(None) => Err(NotesError::NoteNotFound),
            Err(_) => Err(NotesError::DbError),
        }
    } else {
        Err(NotesError::AuthorNotFound)
    }
}

pub async fn update_note_by_id(
    State(db): State<DB>,
    claims: Claims,
    Path(id): Path<String>,
    Json(payload): Json<CreateNotePayload>,
) -> Result<Json<NoteInfo>, NotesError> {
    let user = match db.get_user(claims.email).await {
        Some(user) => {
            user
        }, 
        None => {
            return Err(NotesError::AuthorNotFound)
        }
    }; 
        let filter = doc! {"_id": ObjectId::from_str(&id).unwrap(), "client_id": user.id, };
        println!("{:?}", filter);
        match db
            .notes
            .find_one_and_update(
                filter,
                doc! {"$set": {
                    "title": payload.title,
                    "content": payload.content,
                    "tags": payload.tags
                }},
            )
            .await
        {
            Ok(Some(res)) => Ok(Json(res)),
            Ok(None) => Err(NotesError::NoteNotFound),
            Err(err) => {
                println!("{:?}", err);
                Err(NotesError::DbError)
            }
        }
    
}
