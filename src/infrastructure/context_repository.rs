use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Context, ContextId, ContextName, UserId};

struct ContextRecord {
    id: Uuid,
    user_id: Uuid,
    name: String,
    position: i32,
    created_at: DateTime<Utc>,
}

impl ContextRecord {
    fn into_domain(self) -> Context {
        let id = ContextId::from_uuid(self.id);
        let user_id = UserId::from_uuid(self.user_id);
        let name = ContextName::parse(self.name).expect("stored context name should be valid");

        Context::from_parts(id, user_id, name, self.position, self.created_at)
    }
}

pub async fn insert_context(pool: &PgPool, context: &Context) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO contexts (id, user_id, name, position, created_at)
           VALUES ($1, $2, $3, $4, $5 :: timestamptz)"#,
        context.id().as_uuid(),
        context.user_id().as_uuid(),
        context.name().as_str(),
        context.position(),
        context.created_at() as &DateTime<Utc>,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_contexts_by_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<Context>, sqlx::Error> {
    let records = sqlx::query_as!(
        ContextRecord,
        r#"SELECT id, user_id, name, position,
           created_at as "created_at: DateTime<Utc>"
           FROM contexts WHERE user_id = $1 ORDER BY position ASC, name ASC"#,
        user_id.as_uuid(),
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|r: ContextRecord| r.into_domain())
        .collect())
}

pub async fn find_context_by_id(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Option<Context>, sqlx::Error> {
    let record: Option<ContextRecord> = sqlx::query_as!(
        ContextRecord,
        r#"SELECT id, user_id, name, position,
           created_at as "created_at: DateTime<Utc>"
           FROM contexts WHERE id = $1"#,
        context_id.as_uuid(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.map(|r: ContextRecord| r.into_domain()))
}

pub async fn update_context_name(
    pool: &PgPool,
    context_id: &ContextId,
    name: &ContextName,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE contexts SET name = $1 WHERE id = $2"#,
        name.as_str(),
        context_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_context(pool: &PgPool, context_id: &ContextId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM contexts WHERE id = $1"#,
        context_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_max_position(pool: &PgPool, user_id: &UserId) -> Result<i32, sqlx::Error> {
    let record = sqlx::query!(
        r#"SELECT COALESCE(MAX(position), -1) as "max_pos!" FROM contexts WHERE user_id = $1"#,
        user_id.as_uuid(),
    )
    .fetch_one(pool)
    .await?;

    Ok(record.max_pos)
}
