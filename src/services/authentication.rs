use sqlx::PgPool;

use crate::domain::{PasswordHash_, UserId, ValidatedEmail};
use crate::infrastructure::user_repository;

#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "authenticate_user",
    skip(pool, password),
    fields(user_email = %email)
)]
pub async fn authenticate_user(
    pool: &PgPool,
    email: String,
    password: String,
) -> Result<UserId, AuthenticationError> {
    let validated_email =
        ValidatedEmail::parse(email).map_err(|_| AuthenticationError::InvalidCredentials)?;

    let user_record = user_repository::find_user_by_email(pool, validated_email.as_str())
        .await
        .map_err(|e| AuthenticationError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(AuthenticationError::InvalidCredentials)?;

    let password_hash = PasswordHash_::from_phc(user_record.password_hash);

    if !password_hash.verify(&password) {
        tracing::warn!("Failed login attempt");
        return Err(AuthenticationError::InvalidCredentials);
    }

    tracing::info!("User authenticated successfully");
    Ok(UserId::from_uuid(user_record.id))
}
