use crate::{
    auth::{generate_acces_token, generate_refresh_token, AuthResponseBody},
    error::ApiError,
    models::user::User,
    repository::user_repo::UserRepo,
};
use bcrypt::*;
use mongodb::bson::oid::ObjectId;
use tracing::error;
pub async fn register_user<R: UserRepo>(
    repo: &R,
    username: &str,
    email: &str,
    password: &str,
) -> Result<AuthResponseBody, ApiError> {
    if (repo.user_exist(username, password)).await? {
        return Err(ApiError::UserExist);
    }

    let hashed_password = hash(password, DEFAULT_COST).map_err(|err| {
        error!("{}", err);
        ApiError::InternalError
    })?;

    let user = User {
        id: ObjectId::new(),
        username: username.to_string(),
        email: email.to_string(),
        password: hashed_password,
    };

    repo.create_user(&user).await?;

    let token = generate_acces_token(username)?;
    let refresh_token = generate_refresh_token(username)?;
    Ok(AuthResponseBody::new(
        token,
        refresh_token,
        username.to_string(),
    ))
}

pub async fn login_user<R: UserRepo>(
    repo: &R,
    username: &str,
    password: &str,
) -> Result<AuthResponseBody, ApiError> {
    let user = repo.get_user(username).await?;

    match verify(password, &user.password) {
        Ok(true) => {
            let token = generate_acces_token(&user.username)?;
            let refresh_token = generate_refresh_token(&user.username)?;
            Ok(AuthResponseBody::new(token, refresh_token, user.username))
        }
        Ok(false) => Err(ApiError::Unathorized),
        Err(err) => {
            error!("{}", err);
            Err(ApiError::InternalError)
        }
    }
}
