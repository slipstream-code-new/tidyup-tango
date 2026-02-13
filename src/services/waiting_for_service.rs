use sqlx::PgPool;

use crate::domain::{
    TodoTitle, TodoTitleError, UserId, WaitingForId, WaitingForItem, WaitingOn, WaitingOnError,
};
use crate::infrastructure::waiting_for_repository;

#[derive(Debug, thiserror::Error)]
pub enum AddWaitingForError {
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error("Invalid waiting-on: {0}")]
    InvalidWaitingOn(#[from] WaitingOnError),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

#[tracing::instrument(
    name = "add_waiting_for_item",
    skip(pool, title, waiting_on),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn add_waiting_for_item(
    pool: &PgPool,
    user_id: UserId,
    title: String,
    waiting_on: String,
) -> Result<WaitingForItem, AddWaitingForError> {
    let title = TodoTitle::parse(title)?;
    let waiting_on = WaitingOn::parse(waiting_on)?;
    let item = WaitingForItem::new(user_id, title, waiting_on);

    waiting_for_repository::insert_waiting_for_item(pool, &item)
        .await
        .map_err(|e| {
            AddWaitingForError::Unexpected(anyhow::anyhow!(
                "Failed to insert waiting-for item: {e}"
            ))
        })?;

    tracing::info!("Waiting-for item added");
    Ok(item)
}

#[tracing::instrument(
    name = "list_active_waiting_for_items",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn list_active_waiting_for_items(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<WaitingForItem>, anyhow::Error> {
    let items = waiting_for_repository::find_active_waiting_for_items_by_user(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch waiting-for items: {e}"))?;

    Ok(items)
}

#[tracing::instrument(
    name = "count_active_waiting_for_items",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn count_active_waiting_for_items(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<i64, anyhow::Error> {
    let count = waiting_for_repository::count_active_waiting_for_items(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to count waiting-for items: {e}"))?;

    Ok(count)
}

#[derive(Debug, thiserror::Error)]
pub enum ResolveWaitingForError {
    #[error("Waiting-for item not found")]
    NotFound,
    #[error("Not authorized to resolve this waiting-for item")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "resolve_waiting_for_item",
    skip(pool),
    fields(item_id = %item_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn resolve_waiting_for_item(
    pool: &PgPool,
    item_id: &WaitingForId,
    user_id: &UserId,
) -> Result<(), ResolveWaitingForError> {
    let item = waiting_for_repository::find_waiting_for_item_by_id(pool, item_id)
        .await
        .map_err(|e| ResolveWaitingForError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(ResolveWaitingForError::NotFound)?;

    if item.user_id() != user_id {
        return Err(ResolveWaitingForError::Unauthorized);
    }

    let resolved = item.resolve();
    if let WaitingForItem::Resolved { resolved_at, .. } = &resolved {
        waiting_for_repository::resolve_waiting_for_item(pool, item_id, resolved_at)
            .await
            .map_err(|e| {
                ResolveWaitingForError::Unexpected(anyhow::anyhow!(
                    "Failed to resolve waiting-for item: {e}"
                ))
            })?;
    }

    tracing::info!("Waiting-for item resolved");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteWaitingForError {
    #[error("Waiting-for item not found")]
    NotFound,
    #[error("Not authorized to delete this waiting-for item")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "delete_waiting_for_item",
    skip(pool),
    fields(item_id = %item_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn delete_waiting_for_item(
    pool: &PgPool,
    item_id: &WaitingForId,
    user_id: &UserId,
) -> Result<(), DeleteWaitingForError> {
    let item = waiting_for_repository::find_waiting_for_item_by_id(pool, item_id)
        .await
        .map_err(|e| DeleteWaitingForError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(DeleteWaitingForError::NotFound)?;

    if item.user_id() != user_id {
        return Err(DeleteWaitingForError::Unauthorized);
    }

    waiting_for_repository::delete_waiting_for_item(pool, item_id)
        .await
        .map_err(|e| {
            DeleteWaitingForError::Unexpected(anyhow::anyhow!(
                "Failed to delete waiting-for item: {e}"
            ))
        })?;

    tracing::info!("Waiting-for item deleted");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateWaitingForError {
    #[error("Waiting-for item not found")]
    NotFound,
    #[error("Not authorized to edit this waiting-for item")]
    Unauthorized,
    #[error("Invalid title: {0}")]
    InvalidTitle(TodoTitleError),
    #[error("Invalid waiting-on: {0}")]
    InvalidWaitingOn(WaitingOnError),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

#[tracing::instrument(
    name = "update_waiting_for_item",
    skip(pool, new_title, new_waiting_on),
    fields(item_id = %item_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn update_waiting_for_item(
    pool: &PgPool,
    item_id: &WaitingForId,
    user_id: &UserId,
    new_title: String,
    new_waiting_on: String,
) -> Result<(), UpdateWaitingForError> {
    let item = waiting_for_repository::find_waiting_for_item_by_id(pool, item_id)
        .await
        .map_err(|e| UpdateWaitingForError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(UpdateWaitingForError::NotFound)?;

    if item.user_id() != user_id {
        return Err(UpdateWaitingForError::Unauthorized);
    }

    let title = TodoTitle::parse(new_title).map_err(UpdateWaitingForError::InvalidTitle)?;
    let waiting_on =
        WaitingOn::parse(new_waiting_on).map_err(UpdateWaitingForError::InvalidWaitingOn)?;

    waiting_for_repository::update_waiting_for_item(pool, item_id, &title, &waiting_on)
        .await
        .map_err(|e| {
            UpdateWaitingForError::Unexpected(anyhow::anyhow!(
                "Failed to update waiting-for item: {e}"
            ))
        })?;

    tracing::info!("Waiting-for item updated");
    Ok(())
}
