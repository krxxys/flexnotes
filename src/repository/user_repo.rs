use crate::error::ApiError;
use crate::models::user::User;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::Collection;
use tracing::error;
#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn get_user(&self, username: &str) -> Result<User, ApiError>;
    async fn user_exist(&self, username: &str, email: &str) -> Result<bool, ApiError>;
    async fn create_user(&self, user: &User) -> Result<User, ApiError>;
}

pub struct MongoUserRepo {
    collection: Collection<User>,
}
impl MongoUserRepo {
    pub fn new(collection: Collection<User>) -> Self {
        Self { collection }
    }
}
#[async_trait]
impl UserRepo for MongoUserRepo {
    async fn get_user(&self, username: &str) -> Result<User, ApiError> {
        let filter = doc! {"username": username};
        match self.collection.find_one(filter).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(ApiError::NotFound),
            Err(err) => {
                error!("{}", err);
                return Err(ApiError::InternalError);
            }
        }
    }

    async fn user_exist(&self, username: &str, email: &str) -> Result<bool, ApiError> {
        //TODO think about the returns
        match self
            .collection
            .find_one(doc! {"$or": [{"username": username}, {"email": email}]})
            .await
        {
            Ok(Some(_)) => Err(ApiError::UserExist),
            Ok(None) => Ok(false),
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }

    async fn create_user(&self, user: &User) -> Result<User, ApiError> {
        match self.collection.insert_one(user).await {
            Ok(_res) => {
                println!("User created");
                Ok(user.to_owned())
            }
            Err(err) => {
                error!("{}", err);
                Err(ApiError::InternalError)
            }
        }
    }
}
