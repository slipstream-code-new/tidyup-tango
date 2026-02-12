use sqlx::PgPool;

use crate::domain::{ContextId, NextAction, Project, ProjectId, TodoTitle, TodoTitleError, UserId};
use crate::infrastructure::{next_action_repository, project_repository};

#[derive(Debug, thiserror::Error)]
pub enum AddProjectError {
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "add_project",
    skip(pool, title),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn add_project(
    pool: &PgPool,
    user_id: UserId,
    title: String,
) -> Result<Project, AddProjectError> {
    let title = TodoTitle::parse(title)?;
    let project = Project::new(user_id, title);

    project_repository::insert_project(pool, &project)
        .await
        .map_err(|e| {
            AddProjectError::Unexpected(anyhow::anyhow!("Failed to insert project: {e}"))
        })?;

    tracing::info!("Project added");
    Ok(project)
}

#[tracing::instrument(
    name = "list_active_projects",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn list_active_projects(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<Project>, anyhow::Error> {
    let projects = project_repository::find_active_projects_by_user(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch projects: {e}"))?;

    Ok(projects)
}

#[tracing::instrument(
    name = "count_active_projects",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn count_active_projects(pool: &PgPool, user_id: &UserId) -> Result<i64, anyhow::Error> {
    let count = project_repository::count_active_projects(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to count projects: {e}"))?;

    Ok(count)
}

#[tracing::instrument(
    name = "count_stalled_projects",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn count_stalled_projects(pool: &PgPool, user_id: &UserId) -> Result<i64, anyhow::Error> {
    let count = project_repository::count_stalled_projects(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to count stalled projects: {e}"))?;

    Ok(count)
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteProjectError {
    #[error("Project not found")]
    NotFound,
    #[error("Not authorized to complete this project")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "complete_project",
    skip(pool),
    fields(project_id = %project_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn complete_project(
    pool: &PgPool,
    project_id: &ProjectId,
    user_id: &UserId,
) -> Result<(), CompleteProjectError> {
    let project = project_repository::find_project_by_id(pool, project_id)
        .await
        .map_err(|e| CompleteProjectError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(CompleteProjectError::NotFound)?;

    if project.user_id() != user_id {
        return Err(CompleteProjectError::Unauthorized);
    }

    let completed = project.complete();
    if let Project::Completed { completed_at, .. } = &completed {
        project_repository::complete_project(pool, project_id, completed_at)
            .await
            .map_err(|e| {
                CompleteProjectError::Unexpected(anyhow::anyhow!("Failed to complete project: {e}"))
            })?;
    }

    tracing::info!("Project completed");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteProjectError {
    #[error("Project not found")]
    NotFound,
    #[error("Not authorized to delete this project")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "delete_project",
    skip(pool),
    fields(project_id = %project_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn delete_project(
    pool: &PgPool,
    project_id: &ProjectId,
    user_id: &UserId,
) -> Result<(), DeleteProjectError> {
    let project = project_repository::find_project_by_id(pool, project_id)
        .await
        .map_err(|e| DeleteProjectError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(DeleteProjectError::NotFound)?;

    if project.user_id() != user_id {
        return Err(DeleteProjectError::Unauthorized);
    }

    project_repository::delete_project(pool, project_id)
        .await
        .map_err(|e| {
            DeleteProjectError::Unexpected(anyhow::anyhow!("Failed to delete project: {e}"))
        })?;

    tracing::info!("Project deleted");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateProjectTitleError {
    #[error("Project not found")]
    NotFound,
    #[error("Not authorized to edit this project")]
    Unauthorized,
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "update_project_title",
    skip(pool, new_title),
    fields(project_id = %project_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn update_project_title(
    pool: &PgPool,
    project_id: &ProjectId,
    user_id: &UserId,
    new_title: String,
) -> Result<(), UpdateProjectTitleError> {
    let project = project_repository::find_project_by_id(pool, project_id)
        .await
        .map_err(|e| UpdateProjectTitleError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(UpdateProjectTitleError::NotFound)?;

    if project.user_id() != user_id {
        return Err(UpdateProjectTitleError::Unauthorized);
    }

    let title = TodoTitle::parse(new_title)?;

    // We need an update_project_title repo function
    sqlx::query!(
        r#"UPDATE projects SET title = $1 WHERE id = $2"#,
        title.as_str(),
        project_id.as_uuid(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        UpdateProjectTitleError::Unexpected(anyhow::anyhow!("Failed to update project title: {e}"))
    })?;

    tracing::info!("Project title updated");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum GetProjectError {
    #[error("Project not found")]
    NotFound,
    #[error("Not authorized to view this project")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub struct ProjectDetail {
    pub project: Project,
    pub next_actions: Vec<NextAction>,
    pub active_next_action_count: i64,
}

#[tracing::instrument(
    name = "get_project_detail",
    skip(pool),
    fields(project_id = %project_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn get_project_detail(
    pool: &PgPool,
    project_id: &ProjectId,
    user_id: &UserId,
) -> Result<ProjectDetail, GetProjectError> {
    let project = project_repository::find_project_by_id(pool, project_id)
        .await
        .map_err(|e| GetProjectError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(GetProjectError::NotFound)?;

    if project.user_id() != user_id {
        return Err(GetProjectError::Unauthorized);
    }

    let next_actions = next_action_repository::find_next_actions_by_project(pool, project_id)
        .await
        .map_err(|e| {
            GetProjectError::Unexpected(anyhow::anyhow!("Failed to fetch next actions: {e}"))
        })?;

    let active_next_action_count = project_repository::count_project_next_actions(pool, project_id)
        .await
        .map_err(|e| {
            GetProjectError::Unexpected(anyhow::anyhow!("Failed to count next actions: {e}"))
        })?;

    Ok(ProjectDetail {
        project,
        next_actions,
        active_next_action_count,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum AddProjectNextActionError {
    #[error("Project not found")]
    ProjectNotFound,
    #[error("Not authorized")]
    Unauthorized,
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "add_next_action_to_project",
    skip(pool, title),
    fields(project_id = %project_id.as_uuid(), user_id = %user_id.as_uuid(), context_id = %context_id.as_uuid())
)]
pub async fn add_next_action_to_project(
    pool: &PgPool,
    user_id: &UserId,
    project_id: &ProjectId,
    context_id: ContextId,
    title: String,
) -> Result<NextAction, AddProjectNextActionError> {
    let project = project_repository::find_project_by_id(pool, project_id)
        .await
        .map_err(|e| AddProjectNextActionError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(AddProjectNextActionError::ProjectNotFound)?;

    if project.user_id() != user_id {
        return Err(AddProjectNextActionError::Unauthorized);
    }

    let title = TodoTitle::parse(title)?;
    let action =
        NextAction::new_for_project(user_id.clone(), context_id, project_id.clone(), title);

    next_action_repository::insert_next_action(pool, &action)
        .await
        .map_err(|e| {
            AddProjectNextActionError::Unexpected(anyhow::anyhow!(
                "Failed to insert next action: {e}"
            ))
        })?;

    tracing::info!("Next action added to project");
    Ok(action)
}
