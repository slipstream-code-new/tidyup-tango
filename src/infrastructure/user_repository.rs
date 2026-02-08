use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{PasswordHash_, UserId, ValidatedEmail};

pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
}

pub async fn insert_user(
    pool: &PgPool,
    id: &UserId,
    email: &ValidatedEmail,
    password_hash: &PasswordHash_,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)"#,
        id.as_uuid(),
        email.as_str(),
        password_hash.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let record = sqlx::query_as!(
        UserRecord,
        r#"SELECT id, email, password_hash FROM users WHERE email = $1"#,
        email,
    )
    .fetch_optional(pool)
    .await?;
    Ok(record)
}
