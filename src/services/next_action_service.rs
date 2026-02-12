use sqlx::PgPool;

use crate::domain::{ContextId, NextAction, NextActionId, TodoTitle, TodoTitleError, UserId};
use crate::infrastructure::next_action_repository;

#[derive(Debug, thiserror::Error)]
pub enum AddNextActionError {
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "add_next_action",
    skip(pool, title),
    fields(user_id = %user_id.as_uuid(), context_id = %context_id.as_uuid())
)]
pub async fn add_next_action(
    pool: &PgPool,
    user_id: UserId,
    context_id: ContextId,
    title: String,
) -> Result<NextAction, AddNextActionError> {
    let title = TodoTitle::parse(title)?;
    let action = NextAction::new(user_id, context_id, title);

    next_action_repository::insert_next_action(pool, &action)
        .await
        .map_err(|e| {
            AddNextActionError::Unexpected(anyhow::anyhow!("Failed to insert next action: {e}"))
        })?;

    tracing::info!("Next action added");
    Ok(action)
}

#[tracing::instrument(
    name = "list_active_next_actions",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn list_active_next_actions(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<NextAction>, anyhow::Error> {
    let actions = next_action_repository::find_active_next_actions_by_user(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch next actions: {e}"))?;

    Ok(actions)
}

#[tracing::instrument(
    name = "list_active_next_actions_by_context",
    skip(pool),
    fields(user_id = %user_id.as_uuid(), context_id = %context_id.as_uuid())
)]
pub async fn list_active_next_actions_by_context(
    pool: &PgPool,
    user_id: &UserId,
    context_id: &ContextId,
) -> Result<Vec<NextAction>, anyhow::Error> {
    let actions = next_action_repository::find_active_next_actions_by_user_and_context(
        pool, user_id, context_id,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to fetch next actions by context: {e}"))?;

    Ok(actions)
}

#[tracing::instrument(
    name = "count_active_next_actions",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn count_active_next_actions(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<i64, anyhow::Error> {
    let count = next_action_repository::count_active_next_actions(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to count next actions: {e}"))?;

    Ok(count)
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteNextActionError {
    #[error("Next action not found")]
    NotFound,
    #[error("Not authorized to complete this next action")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "complete_next_action",
    skip(pool),
    fields(action_id = %action_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn complete_next_action(
    pool: &PgPool,
    action_id: &NextActionId,
    user_id: &UserId,
) -> Result<(), CompleteNextActionError> {
    let action = next_action_repository::find_next_action_by_id(pool, action_id)
        .await
        .map_err(|e| CompleteNextActionError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(CompleteNextActionError::NotFound)?;

    if action.user_id() != user_id {
        return Err(CompleteNextActionError::Unauthorized);
    }

    let completed = action.complete();
    if let NextAction::Completed { completed_at, .. } = &completed {
        next_action_repository::complete_next_action(pool, action_id, completed_at)
            .await
            .map_err(|e| {
                CompleteNextActionError::Unexpected(anyhow::anyhow!(
                    "Failed to complete next action: {e}"
                ))
            })?;
    }

    tracing::info!("Next action completed");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteNextActionError {
    #[error("Next action not found")]
    NotFound,
    #[error("Not authorized to delete this next action")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "delete_next_action",
    skip(pool),
    fields(action_id = %action_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn delete_next_action(
    pool: &PgPool,
    action_id: &NextActionId,
    user_id: &UserId,
) -> Result<(), DeleteNextActionError> {
    let action = next_action_repository::find_next_action_by_id(pool, action_id)
        .await
        .map_err(|e| DeleteNextActionError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(DeleteNextActionError::NotFound)?;

    if action.user_id() != user_id {
        return Err(DeleteNextActionError::Unauthorized);
    }

    next_action_repository::delete_next_action(pool, action_id)
        .await
        .map_err(|e| {
            DeleteNextActionError::Unexpected(anyhow::anyhow!("Failed to delete next action: {e}"))
        })?;

    tracing::info!("Next action deleted");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateNextActionTitleError {
    #[error("Next action not found")]
    NotFound,
    #[error("Not authorized to edit this next action")]
    Unauthorized,
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "update_next_action_title",
    skip(pool, new_title),
    fields(action_id = %action_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn update_next_action_title(
    pool: &PgPool,
    action_id: &NextActionId,
    user_id: &UserId,
    new_title: String,
) -> Result<(), UpdateNextActionTitleError> {
    let action = next_action_repository::find_next_action_by_id(pool, action_id)
        .await
        .map_err(|e| {
            UpdateNextActionTitleError::Unexpected(anyhow::anyhow!("Database error: {e}"))
        })?
        .ok_or(UpdateNextActionTitleError::NotFound)?;

    if action.user_id() != user_id {
        return Err(UpdateNextActionTitleError::Unauthorized);
    }

    let title = TodoTitle::parse(new_title)?;

    next_action_repository::update_next_action_title(pool, action_id, &title)
        .await
        .map_err(|e| {
            UpdateNextActionTitleError::Unexpected(anyhow::anyhow!(
                "Failed to update next action title: {e}"
            ))
        })?;

    tracing::info!("Next action title updated");
    Ok(())
}
