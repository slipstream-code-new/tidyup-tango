use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::domain::{SomedayMaybeId, SomedayMaybeItem, TodoTitle, UserId};

struct SomedayMaybeRecord {
    id: Uuid,
    user_id: Uuid,
    title: String,
    created_at: DateTime<Utc>,
}

impl SomedayMaybeRecord {
    fn into_domain(self) -> SomedayMaybeItem {
        let id = SomedayMaybeId::from_uuid(self.id);
        let user_id = UserId::from_uuid(self.user_id);
        let title = TodoTitle::parse(self.title).expect("stored title should be valid");

        SomedayMaybeItem::from_parts(id, user_id, title, self.created_at)
    }
}

pub async fn insert_someday_maybe_item(
    executor: impl PgExecutor<'_>,
    item: &SomedayMaybeItem,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO someday_maybe_items (id, user_id, title, created_at)
           VALUES ($1, $2, $3, $4 :: timestamptz)"#,
        item.id().as_uuid(),
        item.user_id().as_uuid(),
        item.title().as_str(),
        item.created_at() as &DateTime<Utc>,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn find_someday_maybe_items_by_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<SomedayMaybeItem>, sqlx::Error> {
    let records = sqlx::query_as!(
        SomedayMaybeRecord,
        r#"SELECT id, user_id, title,
           created_at as "created_at: DateTime<Utc>"
           FROM someday_maybe_items
           WHERE user_id = $1
           ORDER BY created_at ASC"#,
        user_id.as_uuid(),
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|r: SomedayMaybeRecord| r.into_domain())
        .collect())
}

pub async fn find_someday_maybe_item_by_id(
    executor: impl PgExecutor<'_>,
    item_id: &SomedayMaybeId,
) -> Result<Option<SomedayMaybeItem>, sqlx::Error> {
    let record: Option<SomedayMaybeRecord> = sqlx::query_as!(
        SomedayMaybeRecord,
        r#"SELECT id, user_id, title,
           created_at as "created_at: DateTime<Utc>"
           FROM someday_maybe_items WHERE id = $1"#,
        item_id.as_uuid(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(record.map(|r| r.into_domain()))
}

pub async fn delete_someday_maybe_item(
    executor: impl PgExecutor<'_>,
    item_id: &SomedayMaybeId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM someday_maybe_items WHERE id = $1"#,
        item_id.as_uuid(),
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn count_someday_maybe_items(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!(
        r#"SELECT COUNT(*) as "count!" FROM someday_maybe_items WHERE user_id = $1"#,
        user_id.as_uuid(),
    )
    .fetch_one(pool)
    .await?;

    Ok(record.count)
}

pub async fn update_someday_maybe_title(
    pool: &PgPool,
    item_id: &SomedayMaybeId,
    title: &TodoTitle,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE someday_maybe_items SET title = $1 WHERE id = $2"#,
        title.as_str(),
        item_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}
