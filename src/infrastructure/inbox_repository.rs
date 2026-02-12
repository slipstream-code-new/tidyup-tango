use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::domain::{InboxItem, InboxItemId, TodoTitle, UserId};

struct InboxRecord {
    id: Uuid,
    user_id: Uuid,
    title: String,
    created_at: DateTime<Utc>,
}

impl InboxRecord {
    fn into_domain(self) -> InboxItem {
        let id = InboxItemId::from_uuid(self.id);
        let user_id = UserId::from_uuid(self.user_id);
        let title = TodoTitle::parse(self.title).expect("stored title should be valid");

        InboxItem::from_parts(id, user_id, title, self.created_at)
    }
}

pub async fn insert_inbox_item(pool: &PgPool, item: &InboxItem) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO inbox_items (id, user_id, title, created_at)
           VALUES ($1, $2, $3, $4 :: timestamptz)"#,
        item.id().as_uuid(),
        item.user_id().as_uuid(),
        item.title().as_str(),
        item.created_at() as &DateTime<Utc>,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_inbox_items_by_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<InboxItem>, sqlx::Error> {
    let records = sqlx::query_as!(
        InboxRecord,
        r#"SELECT id, user_id, title,
           created_at as "created_at: DateTime<Utc>"
           FROM inbox_items WHERE user_id = $1 ORDER BY created_at ASC"#,
        user_id.as_uuid(),
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|r: InboxRecord| r.into_domain())
        .collect())
}

pub async fn count_inbox_items(pool: &PgPool, user_id: &UserId) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!(
        r#"SELECT COUNT(*) as "count!" FROM inbox_items WHERE user_id = $1"#,
        user_id.as_uuid(),
    )
    .fetch_one(pool)
    .await?;

    Ok(record.count)
}

pub async fn delete_inbox_item(
    executor: impl PgExecutor<'_>,
    item_id: &InboxItemId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM inbox_items WHERE id = $1"#,
        item_id.as_uuid(),
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn find_inbox_item_by_id(
    executor: impl PgExecutor<'_>,
    item_id: &InboxItemId,
) -> Result<Option<InboxItem>, sqlx::Error> {
    let record: Option<InboxRecord> = sqlx::query_as!(
        InboxRecord,
        r#"SELECT id, user_id, title,
           created_at as "created_at: DateTime<Utc>"
           FROM inbox_items WHERE id = $1"#,
        item_id.as_uuid(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(record.map(|r: InboxRecord| r.into_domain()))
}
