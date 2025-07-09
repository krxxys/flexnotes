use crate::{
    logger::{DatabaseLog, MognoDBLogger},
    models::{note::Note, todo::TodoList, user::User},
    repository::{note_repo::MongoNoteRepo, todo_repo::MongoTodoRepo, user_repo::MongoUserRepo},
    MONGO_URL,
};
use mongodb::{options::ClientOptions, Client, Collection};
use std::{sync::Arc, time::Duration};
use tracing::error;

#[derive(Clone)]
pub struct Database {
    users: Collection<User>,
    notes: Collection<Note>,
    todos: Collection<TodoList>,
    logs: Collection<DatabaseLog>,
}

impl Database {
    pub async fn new() -> Self {
        let mut options = ClientOptions::parse(MONGO_URL.clone())
            .await
            .map_err(|err| {
                error!("{}", err);
            })
            .unwrap();
        options.connect_timeout = Some(Duration::from_secs(2));
        options.server_selection_timeout = Some(Duration::from_secs(2));
        /*
        let mongo_client = Client::with_options(options)
            .expect("")
            .await
            .expect("Failed to connect to the mongodb server")
            .database("flexnote");
        */

        let mongo_client = Client::with_options(options)
            .expect("Failed connecting with database")
            .database("flexnote");

        let users_collection = mongo_client.collection::<User>("users");

        let notes_collection = mongo_client.collection::<Note>("notes");

        let todos_collection = mongo_client.collection::<TodoList>("todos");

        let logs_collection = mongo_client.collection::<DatabaseLog>("logs");

        Self {
            users: users_collection,
            notes: notes_collection,
            todos: todos_collection,
            logs: logs_collection,
        }
    }

    pub fn user_repo(&self) -> MongoUserRepo {
        MongoUserRepo::new(self.users.clone())
    }

    pub fn note_repo(&self) -> MongoNoteRepo {
        MongoNoteRepo::new(self.notes.clone())
    }
    pub fn todos_repo(&self) -> MongoTodoRepo {
        MongoTodoRepo::new(self.todos.clone())
    }

    pub fn logs_repo(&self) -> MognoDBLogger {
        MognoDBLogger::new(self.logs.clone())
    }
}
