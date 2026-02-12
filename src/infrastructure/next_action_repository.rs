use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::domain::{ContextId, NextAction, NextActionId, ProjectId, TodoTitle, UserId};

struct NextActionRecord {
    id: Uuid,
    user_id: Uuid,
    context_id: Uuid,
    project_id: Option<Uuid>,
    title: String,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl NextActionRecord {
    fn into_domain(self) -> NextAction {
        let id = NextActionId::from_uuid(self.id);
        let user_id = UserId::from_uuid(self.user_id);
        let context_id = ContextId::from_uuid(self.context_id);
        let project_id = self.project_id.map(ProjectId::from_uuid);
        let title = TodoTitle::parse(self.title).expect("stored title should be valid");

        NextAction::from_parts(
            id,
            user_id,
            context_id,
            project_id,
            title,
            self.created_at,
            self.completed_at,
        )
    }
}

pub async fn insert_next_action(
    executor: impl PgExecutor<'_>,
    action: &NextAction,
) -> Result<(), sqlx::Error> {
    let completed_at: Option<&DateTime<Utc>> = match action {
        NextAction::Completed { completed_at, .. } => Some(completed_at),
        NextAction::Active { .. } => None,
    };

    let project_id = action.project_id().map(|pid| *pid.as_uuid());

    sqlx::query!(
        r#"INSERT INTO next_actions (id, user_id, context_id, project_id, title, created_at, completed_at)
           VALUES ($1, $2, $3, $4, $5, $6 :: timestamptz, $7 :: timestamptz)"#,
        action.id().as_uuid(),
        action.user_id().as_uuid(),
        action.context_id().as_uuid(),
        project_id as Option<Uuid>,
        action.title().as_str(),
        action.created_at() as &DateTime<Utc>,
        completed_at as Option<&DateTime<Utc>>,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn find_active_next_actions_by_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<NextAction>, sqlx::Error> {
    let records = sqlx::query_as!(
        NextActionRecord,
        r#"SELECT id, user_id, context_id, project_id, title,
           created_at as "created_at: DateTime<Utc>",
           completed_at as "completed_at: DateTime<Utc>"
           FROM next_actions
           WHERE user_id = $1 AND completed_at IS NULL
           ORDER BY created_at ASC"#,
        user_id.as_uuid(),
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|r: NextActionRecord| r.into_domain())
        .collect())
}

pub async fn find_active_next_actions_by_user_and_context(
    pool: &PgPool,
    user_id: &UserId,
    context_id: &ContextId,
) -> Result<Vec<NextAction>, sqlx::Error> {
    let records = sqlx::query_as!(
        NextActionRecord,
        r#"SELECT id, user_id, context_id, project_id, title,
           created_at as "created_at: DateTime<Utc>",
           completed_at as "completed_at: DateTime<Utc>"
           FROM next_actions
           WHERE user_id = $1 AND context_id = $2 AND completed_at IS NULL
           ORDER BY created_at ASC"#,
        user_id.as_uuid(),
        context_id.as_uuid(),
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|r: NextActionRecord| r.into_domain())
        .collect())
}

pub async fn find_next_action_by_id(
    pool: &PgPool,
    action_id: &NextActionId,
) -> Result<Option<NextAction>, sqlx::Error> {
    let record: Option<NextActionRecord> = sqlx::query_as!(
        NextActionRecord,
        r#"SELECT id, user_id, context_id, project_id, title,
           created_at as "created_at: DateTime<Utc>",
           completed_at as "completed_at: DateTime<Utc>"
           FROM next_actions WHERE id = $1"#,
        action_id.as_uuid(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.map(|r: NextActionRecord| r.into_domain()))
}

pub async fn complete_next_action(
    pool: &PgPool,
    action_id: &NextActionId,
    completed_at: &DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE next_actions SET completed_at = $1 :: timestamptz WHERE id = $2"#,
        completed_at as &DateTime<Utc>,
        action_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_next_action(
    pool: &PgPool,
    action_id: &NextActionId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM next_actions WHERE id = $1"#,
        action_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_next_action_title(
    pool: &PgPool,
    action_id: &NextActionId,
    title: &TodoTitle,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE next_actions SET title = $1 WHERE id = $2"#,
        title.as_str(),
        action_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_active_next_actions(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!(
        r#"SELECT COUNT(*) as "count!" FROM next_actions WHERE user_id = $1 AND completed_at IS NULL"#,
        user_id.as_uuid(),
    )
    .fetch_one(pool)
    .await?;

    Ok(record.count)
}

pub async fn find_next_actions_by_project(
    pool: &PgPool,
    project_id: &ProjectId,
) -> Result<Vec<NextAction>, sqlx::Error> {
    let records = sqlx::query_as!(
        NextActionRecord,
        r#"SELECT id, user_id, context_id, project_id, title,
           created_at as "created_at: DateTime<Utc>",
           completed_at as "completed_at: DateTime<Utc>"
           FROM next_actions
           WHERE project_id = $1
           ORDER BY completed_at NULLS FIRST, created_at ASC"#,
        project_id.as_uuid(),
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| r.into_domain()).collect())
}
