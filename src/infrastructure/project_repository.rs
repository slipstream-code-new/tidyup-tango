use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::domain::{Project, ProjectId, TodoTitle, UserId};

struct ProjectRecord {
    id: Uuid,
    user_id: Uuid,
    title: String,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    dropped_at: Option<DateTime<Utc>>,
}

impl ProjectRecord {
    fn into_domain(self) -> Project {
        let id = ProjectId::from_uuid(self.id);
        let user_id = UserId::from_uuid(self.user_id);
        let title = TodoTitle::parse(self.title).expect("stored title should be valid");

        Project::from_parts(
            id,
            user_id,
            title,
            self.created_at,
            self.completed_at,
            self.dropped_at,
        )
    }
}

pub async fn insert_project(
    executor: impl PgExecutor<'_>,
    project: &Project,
) -> Result<(), sqlx::Error> {
    let (completed_at, dropped_at): (Option<&DateTime<Utc>>, Option<&DateTime<Utc>>) = match project
    {
        Project::Completed { completed_at, .. } => (Some(completed_at), None),
        Project::Dropped { dropped_at, .. } => (None, Some(dropped_at)),
        Project::Active { .. } => (None, None),
    };

    sqlx::query!(
        r#"INSERT INTO projects (id, user_id, title, created_at, completed_at, dropped_at)
           VALUES ($1, $2, $3, $4 :: timestamptz, $5 :: timestamptz, $6 :: timestamptz)"#,
        project.id().as_uuid(),
        project.user_id().as_uuid(),
        project.title().as_str(),
        project.created_at() as &DateTime<Utc>,
        completed_at as Option<&DateTime<Utc>>,
        dropped_at as Option<&DateTime<Utc>>,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn find_active_projects_by_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<Project>, sqlx::Error> {
    let records = sqlx::query_as!(
        ProjectRecord,
        r#"SELECT id, user_id, title,
           created_at as "created_at: DateTime<Utc>",
           completed_at as "completed_at: DateTime<Utc>",
           dropped_at as "dropped_at: DateTime<Utc>"
           FROM projects
           WHERE user_id = $1 AND completed_at IS NULL AND dropped_at IS NULL
           ORDER BY created_at ASC"#,
        user_id.as_uuid(),
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| r.into_domain()).collect())
}

pub async fn find_project_by_id(
    executor: impl PgExecutor<'_>,
    project_id: &ProjectId,
) -> Result<Option<Project>, sqlx::Error> {
    let record: Option<ProjectRecord> = sqlx::query_as!(
        ProjectRecord,
        r#"SELECT id, user_id, title,
           created_at as "created_at: DateTime<Utc>",
           completed_at as "completed_at: DateTime<Utc>",
           dropped_at as "dropped_at: DateTime<Utc>"
           FROM projects WHERE id = $1"#,
        project_id.as_uuid(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(record.map(|r| r.into_domain()))
}

pub async fn complete_project(
    pool: &PgPool,
    project_id: &ProjectId,
    completed_at: &DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE projects SET completed_at = $1 :: timestamptz WHERE id = $2"#,
        completed_at as &DateTime<Utc>,
        project_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_project(pool: &PgPool, project_id: &ProjectId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM projects WHERE id = $1"#,
        project_id.as_uuid(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_active_projects(pool: &PgPool, user_id: &UserId) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!(
        r#"SELECT COUNT(*) as "count!" FROM projects WHERE user_id = $1 AND completed_at IS NULL AND dropped_at IS NULL"#,
        user_id.as_uuid(),
    )
    .fetch_one(pool)
    .await?;

    Ok(record.count)
}

/// Count stalled projects (active projects with no active next actions)
pub async fn count_stalled_projects(pool: &PgPool, user_id: &UserId) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!(
        r#"SELECT COUNT(*) as "count!"
           FROM projects p
           WHERE p.user_id = $1
             AND p.completed_at IS NULL
             AND p.dropped_at IS NULL
             AND NOT EXISTS (
                 SELECT 1 FROM next_actions na
                 WHERE na.project_id = p.id
                   AND na.completed_at IS NULL
             )"#,
        user_id.as_uuid(),
    )
    .fetch_one(pool)
    .await?;

    Ok(record.count)
}

/// Count active next actions for a specific project
pub async fn count_project_next_actions(
    pool: &PgPool,
    project_id: &ProjectId,
) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!(
        r#"SELECT COUNT(*) as "count!" FROM next_actions WHERE project_id = $1 AND completed_at IS NULL"#,
        project_id.as_uuid(),
    )
    .fetch_one(pool)
    .await?;

    Ok(record.count)
}
