use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/*
* Note <-> TodoList <-> Todo
*/

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoList {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: ObjectId,
    pub todos: Vec<Todo>,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Todo {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub title: String,
    pub status: bool,
    pub priority: TodoPriority,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum TodoPriority {
    High,
    Normal,
    Low,
}
