use sqlx::PgPool;

use crate::domain::{EmailValidationError, Password, PasswordError, UserId, ValidatedEmail};
use crate::infrastructure::user_repository;

#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("Invalid email: {0}")]
    InvalidEmail(#[from] EmailValidationError),
    #[error("Invalid password: {0}")]
    InvalidPassword(#[from] PasswordError),
    #[error("Passwords do not match")]
    PasswordMismatch,
    #[error("An account with this email already exists")]
    DuplicateEmail,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "register_user",
    skip(pool, password, password_confirm),
    fields(user_email = %email)
)]
pub async fn register_user(
    pool: &PgPool,
    email: String,
    password: String,
    password_confirm: String,
) -> Result<UserId, RegistrationError> {
    let validated_email = ValidatedEmail::parse(email)?;

    if password != password_confirm {
        return Err(RegistrationError::PasswordMismatch);
    }

    let validated_password = Password::parse(password)?;

    let password_hash = validated_password
        .hash()
        .map_err(RegistrationError::Unexpected)?;

    let user_id = UserId::new();

    user_repository::insert_user(pool, &user_id, &validated_email, &password_hash)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                RegistrationError::DuplicateEmail
            } else {
                RegistrationError::Unexpected(anyhow::anyhow!("Database error: {e}"))
            }
        })?;

    tracing::info!("New user registered");

    Ok(user_id)
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        db_err.code().as_deref() == Some("23505")
    } else {
        false
    }
}
