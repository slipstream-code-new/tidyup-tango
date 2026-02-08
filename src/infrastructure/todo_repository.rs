use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{TodoItem, TodoItemId, TodoTitle, UserId};

pub struct TodoRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TodoRecord {
    pub fn to_domain(self) -> TodoItem {
        let id = TodoItemId::from_uuid(self.id);
        let user_id = UserId::from_uuid(self.user_id);
        // Title was validated on insert, so we trust it here
        let title = TodoTitle::parse(self.title).expect("stored title should be valid");

        match self.completed_at {
            Some(completed_at) => TodoItem::Completed {
                id,
                user_id,
                title,
                created_at: self.created_at,
                completed_at,
            },
            None => TodoItem::Pending {
                id,
                user_id,
                title,
                created_at: self.created_at,
            },
        }
    }
}

pub async fn insert_todo(pool: &PgPool, item: &TodoItem) -> Result<(), sqlx::Error> {
    let (completed_at, created_at) = match item {
        TodoItem::Pending { created_at, .. } => (None, *created_at),
        TodoItem::Completed {
            created_at,
            completed_at,
            ..
        } => (Some(*completed_at), *created_at),
    };

    sqlx::query!(
        r#"INSERT INTO todos (id, user_id, title, completed_at, created_at)
           VALUES ($1, $2, $3, $4 :: timestamptz, $5 :: timestamptz)"#,
        item.id().as_uuid(),
        item.user_id().as_uuid(),
        item.title().as_str(),
        completed_at as Option<DateTime<Utc>>,
        created_at as DateTime<Utc>,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_todos_by_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<TodoItem>, sqlx::Error> {
    let records = sqlx::query_as!(
        TodoRecord,
        r#"SELECT id, user_id, title,
           completed_at as "completed_at: DateTime<Utc>",
           created_at as "created_at: DateTime<Utc>"
           FROM todos WHERE user_id = $1 ORDER BY created_at ASC"#,
        user_id.as_uuid(),
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| r.to_domain()).collect())
}
