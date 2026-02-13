use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::domain::{TodoTitle, UserId, WaitingForId, WaitingForItem, WaitingOn};

struct WaitingForRecord {
    id: Uuid,
    user_id: Uuid,
    title: String,
    waiting_on: String,
    created_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
}

impl WaitingForRecord {
    fn into_domain(self) -> WaitingForItem {
        let id = WaitingForId::from_uuid(self.id);
        let user_id = UserId::from_uuid(self.user_id);
        let title = TodoTitle::parse(self.title).expect("stored title should be valid");
        let waiting_on =
            WaitingOn::parse(self.waiting_on).expect("stored waiting_on should be valid");

        WaitingForItem::from_parts(
            id,
            user_id,
            title,
            waiting_on,
            self.created_at,
            self.resolved_at,
        )
    }
}

pub async fn insert_waiting_for_item(
    executor: impl PgExecutor<'_>,
    item: &WaitingForItem,
) -> Result<(), sqlx::Error> {
    let resolved_at: Option<&DateTime<Utc>> = match item {
        WaitingForItem::Resolved { resolved_at, .. } => Some(resolved_at),
        WaitingForItem::Active { .. } => None,
    };

    sqlx::query!(
        r#"INSERT INTO waiting_for_items (id, user_id, title, waiting_on, created_at, resolved_at)
           VALUES ($1, $2, $3, $4, $5 :: timestamptz, $6 :: timestamptz)"#,
        item.id().as_uuid(),
        item.user_id().as_uuid(),
        item.title().as_str(),
        item.waiting_on().as_str(),
        item.created_at() as &DateTime<Utc>,
        resolved_at as Option<&DateTime<Utc>>,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn find_active_waiting_for_items_by_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<WaitingForItem>, sqlx::Error> {
    let records = sqlx::query_as!(
        WaitingForRecord,
        r#"SELECT id, user_id, title, waiting_on,
           created_at as "created_at: DateTime<Utc>",
           resolved_at as "resolved_at: DateTime<Utc>"
           FROM waiting_for_items
           WHERE user_id = $1 AND resolved_at IS NULL
           ORDER BY created_at ASC"#,
        user_id.as_uuid(),
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| r.into_domain()).collect())
}

pub async fn find_waiting_for_item_by_id(
    pool: &PgPool,
    item_id: &WaitingForId,
) -> Result<Option<WaitingForItem>, sqlx::Error> {
    let record: Option<WaitingForRecord> = sqlx::query_as!(
        WaitingForRecord,
        r#"SELECT id, user_id, title, waiting_on,
           created_at as "created_at: DateTime<Utc>",
           resolved_at as "resolved_at: DateTime<Utc>"
           FROM waiting_for_items WHERE id = $1"#,
        item_id.as_uuid(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.map(|r| r.into_domain()))
}

pub async fn resolve_waiting_for_item(
    pool: &PgPool,
    item_id: &WaitingForId,
    resolved_at: &DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE waiting_for_items SET resolved_at = $1 :: timestamptz WHERE id = $2"#,
        resolved_at as &DateTime<Utc>,
        item_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_waiting_for_item(
    pool: &PgPool,
    item_id: &WaitingForId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM waiting_for_items WHERE id = $1"#,
        item_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_active_waiting_for_items(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!(
        r#"SELECT COUNT(*) as "count!" FROM waiting_for_items WHERE user_id = $1 AND resolved_at IS NULL"#,
        user_id.as_uuid(),
    )
    .fetch_one(pool)
    .await?;

    Ok(record.count)
}

pub async fn update_waiting_for_item(
    pool: &PgPool,
    item_id: &WaitingForId,
    title: &TodoTitle,
    waiting_on: &WaitingOn,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE waiting_for_items SET title = $1, waiting_on = $2 WHERE id = $3"#,
        title.as_str(),
        waiting_on.as_str(),
        item_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}
