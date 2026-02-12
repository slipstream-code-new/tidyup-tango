use sqlx::PgPool;

use crate::domain::{
    ContextId, InboxItem, InboxItemId, NextAction, TodoTitle, TodoTitleError, UserId,
};
use crate::infrastructure::inbox_repository;
use crate::infrastructure::next_action_repository;

#[derive(Debug, thiserror::Error)]
pub enum CaptureInboxError {
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "capture_inbox_item",
    skip(pool, title),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn capture_inbox_item(
    pool: &PgPool,
    user_id: UserId,
    title: String,
) -> Result<InboxItem, CaptureInboxError> {
    let title = TodoTitle::parse(title)?;
    let item = InboxItem::new(user_id, title);

    inbox_repository::insert_inbox_item(pool, &item)
        .await
        .map_err(|e| {
            CaptureInboxError::Unexpected(anyhow::anyhow!("Failed to insert inbox item: {e}"))
        })?;

    tracing::info!("Inbox item captured");
    Ok(item)
}

#[tracing::instrument(
    name = "get_inbox_items",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn get_inbox_items(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<InboxItem>, anyhow::Error> {
    let items = inbox_repository::find_inbox_items_by_user(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch inbox items: {e}"))?;

    Ok(items)
}

#[tracing::instrument(
    name = "get_inbox_count",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn get_inbox_count(pool: &PgPool, user_id: &UserId) -> Result<i64, anyhow::Error> {
    let count = inbox_repository::count_inbox_items(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to count inbox items: {e}"))?;

    Ok(count)
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteInboxError {
    #[error("Inbox item not found")]
    NotFound,
    #[error("Not authorized to delete this inbox item")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "delete_inbox_item",
    skip(pool),
    fields(item_id = %item_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn delete_inbox_item(
    pool: &PgPool,
    item_id: &InboxItemId,
    user_id: &UserId,
) -> Result<(), DeleteInboxError> {
    let item = inbox_repository::find_inbox_item_by_id(pool, item_id)
        .await
        .map_err(|e| DeleteInboxError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(DeleteInboxError::NotFound)?;

    if item.user_id() != user_id {
        return Err(DeleteInboxError::Unauthorized);
    }

    inbox_repository::delete_inbox_item(pool, item_id)
        .await
        .map_err(|e| {
            DeleteInboxError::Unexpected(anyhow::anyhow!("Failed to delete inbox item: {e}"))
        })?;

    tracing::info!("Inbox item deleted");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ClarifyAsNextActionError {
    #[error("Inbox item not found")]
    NotFound,
    #[error("Not authorized to clarify this inbox item")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "clarify_as_next_action",
    skip(pool),
    fields(item_id = %item_id.as_uuid(), user_id = %user_id.as_uuid(), context_id = %context_id.as_uuid())
)]
pub async fn clarify_as_next_action(
    pool: &PgPool,
    item_id: &InboxItemId,
    user_id: &UserId,
    context_id: ContextId,
) -> Result<NextAction, ClarifyAsNextActionError> {
    let mut tx = pool.begin().await.map_err(|e| {
        ClarifyAsNextActionError::Unexpected(anyhow::anyhow!("Failed to begin transaction: {e}"))
    })?;

    let item = inbox_repository::find_inbox_item_by_id(&mut *tx, item_id)
        .await
        .map_err(|e| ClarifyAsNextActionError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(ClarifyAsNextActionError::NotFound)?;

    if item.user_id() != user_id {
        return Err(ClarifyAsNextActionError::Unauthorized);
    }

    let action = NextAction::new(user_id.clone(), context_id, item.title().clone());

    next_action_repository::insert_next_action(&mut *tx, &action)
        .await
        .map_err(|e| {
            ClarifyAsNextActionError::Unexpected(anyhow::anyhow!(
                "Failed to insert next action: {e}"
            ))
        })?;

    inbox_repository::delete_inbox_item(&mut *tx, item_id)
        .await
        .map_err(|e| {
            ClarifyAsNextActionError::Unexpected(anyhow::anyhow!(
                "Failed to delete inbox item: {e}"
            ))
        })?;

    tx.commit().await.map_err(|e| {
        ClarifyAsNextActionError::Unexpected(anyhow::anyhow!("Failed to commit transaction: {e}"))
    })?;

    tracing::info!("Inbox item clarified as next action");
    Ok(action)
}
