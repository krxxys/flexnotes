use crate::auth::{AuthError, Claims};

pub async fn protected(claims: Claims) -> Result<String, AuthError> {
    Ok(format!("Welcome: {:?}", claims))
}
