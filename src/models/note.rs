use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: ObjectId,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub todo_lists: Vec<ObjectId>,
}
