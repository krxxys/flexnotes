use std::os::linux::raw::stat;

use crate::{
    error::ApiError,
    models::todo::{Todo, TodoList, TodoPriority},
};
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, SerializerOptions},
    Collection,
};
use tracing::error;

#[async_trait]
pub trait TodoRepo: Send + Sync {
    async fn get_todo_lists(
        &self,
        list: Vec<ObjectId>,
        user_id: ObjectId,
    ) -> Result<Vec<TodoList>, ApiError>;
    async fn get_all_todo_lists(&self, user_id: ObjectId) -> Result<Vec<TodoList>, ApiError>;
    async fn create_todo_list(
        &self,
        title: String,
        user_id: ObjectId,
    ) -> Result<TodoList, ApiError>;
    async fn get_todo_list(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<TodoList, ApiError>;
    async fn delete_todo_list(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<(), ApiError>;
    async fn rename_todo_list(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
        title: String,
    ) -> Result<(), ApiError>;
    async fn create_todo(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
        title: String,
        status: bool,
        priority: TodoPriority,
    ) -> Result<(), ApiError>;
    async fn modify_todo(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
        todo_id: ObjectId,
        title: String,
        status: bool,
        priority: TodoPriority,
    ) -> Result<(), ApiError>;
    async fn delete_todo(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
        todo_id: ObjectId,
    ) -> Result<(), ApiError>;
}

pub struct MongoTodoRepo {
    collection: Collection<TodoList>,
}

impl MongoTodoRepo {
    pub fn new(collection: Collection<TodoList>) -> Self {
        Self { collection }
    }
}

#[async_trait]
impl TodoRepo for MongoTodoRepo {
    async fn create_todo_list(
        &self,
        title: String,
        user_id: ObjectId,
    ) -> Result<TodoList, ApiError> {
        let new_todo_list = TodoList {
            id: ObjectId::new(),
            user_id,
            title,
            todos: vec![],
        };
        match self.collection.insert_one(&new_todo_list).await {
            Ok(_res) => Ok(new_todo_list),
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }

    async fn get_all_todo_lists(&self, user_id: ObjectId) -> Result<Vec<TodoList>, ApiError> {
        match self.collection.find(doc! {"user_id": user_id}).await {
            Ok(res) => {
                let todos: Vec<TodoList> = res.try_collect().await.map_err(|err| {
                    error!("{}", err);
                    ApiError::InternalError
                })?;
                if !todos.is_empty() {
                    return Ok(todos);
                }
                Err(ApiError::NotFound)
            }
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }
    async fn get_todo_lists(
        &self,
        list: Vec<ObjectId>,
        user_id: ObjectId,
    ) -> Result<Vec<TodoList>, ApiError> {
        let mut temp: Vec<TodoList> = vec![];
        for id in list.iter() {
            match self
                .collection
                .find_one(doc! { "_id": id, "user_id": user_id})
                .await
            {
                Ok(Some(todo)) => temp.push(todo),
                Ok(None) => {
                    //nothing here is happening
                }
                Err(err) => {
                    error!("{}", err);
                    return Err(ApiError::InternalError);
                }
            }
        }
        Ok(temp)
    }

    async fn delete_todo_list(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<(), ApiError> {
        match self
            .collection
            .delete_one(doc! {
                "_id": todo_list_id,
                "user_id": user_id
            })
            .await
        {
            Ok(res) => {
                if res.deleted_count > 0 {
                    Ok(())
                } else {
                    Err(ApiError::NotFound)
                }
            }
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }

    async fn get_todo_list(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<TodoList, ApiError> {
        match self
            .collection
            .find_one(doc! {"_id": todo_list_id, "user_id": user_id})
            .await
        {
            Ok(Some(todo_list)) => Ok(todo_list),
            Ok(None) => Err(ApiError::NotFound),
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }

    async fn rename_todo_list(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
        title: String,
    ) -> Result<(), ApiError> {
        match self
            .collection
            .find_one(doc! {"_id": todo_list_id, "user_id": user_id})
            .await
        {
            Ok(Some(todo_list)) => {
                match self
                    .collection
                    .update_one(
                        doc! {"_id": todo_list.id},
                        doc! { "$set": { "title": title} },
                    )
                    .await
                {
                    Ok(res) => {
                        if res.modified_count > 0 {
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
            Ok(None) => Err(ApiError::NotFound),
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }

    async fn create_todo(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
        title: String,
        status: bool,
        priority: TodoPriority,
    ) -> Result<(), ApiError> {
        let todo = Todo {
            id: ObjectId::new(),
            title: title.to_string(),
            status,
            priority,
        };

        let todo_doc = bson::to_document(&todo).map_err(|err| {
            error!("{}", err);
            ApiError::InternalError
        })?;

        match self
            .collection
            .update_one(
                doc! {"_id": todo_list_id, "user_id": user_id},
                doc! { "$push" : {
                    "todos": todo_doc
                }},
            )
            .await
        {
            Ok(res) => {
                if res.matched_count > 0 {
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

    async fn modify_todo(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
        todo_id: ObjectId,
        title: String,
        status: bool,
        priority: TodoPriority,
    ) -> Result<(), ApiError> {
        let priority = bson::to_bson_with_options(&priority, SerializerOptions::default())
            .map_err(|err| {
                error!("{}", err);
                ApiError::InternalError
            })?;
        match self.collection.update_one(doc! { "_id": todo_list_id, "user_id": user_id, "todos._id": todo_id}, doc !{"$set": { "todos.$.title": title, "todos.$.status": status, "todos.$.priority": priority}}).await {
            Ok(res) => {
                if res.matched_count > 0 {
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

    async fn delete_todo(
        &self,
        todo_list_id: ObjectId,
        user_id: ObjectId,
        todo_id: ObjectId,
    ) -> Result<(), ApiError> {
        match self
            .collection
            .update_one(
                doc! { "_id": todo_list_id, "user_id": user_id },
                doc! { "$pull": { "todos": { "_id": todo_id}}},
            )
            .await
        {
            Ok(res) => {
                if res.matched_count > 0 {
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
}
