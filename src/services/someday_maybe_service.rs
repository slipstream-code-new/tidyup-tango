use sqlx::PgPool;

use crate::domain::{
    InboxItem, SomedayMaybeId, SomedayMaybeItem, TodoTitle, TodoTitleError, UserId,
};
use crate::infrastructure::inbox_repository;
use crate::infrastructure::someday_maybe_repository;

#[derive(Debug, thiserror::Error)]
pub enum AddSomedayMaybeError {
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

#[tracing::instrument(
    name = "add_someday_maybe_item",
    skip(pool, title),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn add_someday_maybe_item(
    pool: &PgPool,
    user_id: UserId,
    title: String,
) -> Result<SomedayMaybeItem, AddSomedayMaybeError> {
    let title = TodoTitle::parse(title)?;
    let item = SomedayMaybeItem::new(user_id, title);

    someday_maybe_repository::insert_someday_maybe_item(pool, &item)
        .await
        .map_err(|e| {
            AddSomedayMaybeError::Unexpected(anyhow::anyhow!(
                "Failed to insert someday/maybe item: {e}"
            ))
        })?;

    tracing::info!("Someday/maybe item added");
    Ok(item)
}

#[tracing::instrument(
    name = "list_someday_maybe_items",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn list_someday_maybe_items(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<SomedayMaybeItem>, anyhow::Error> {
    let items = someday_maybe_repository::find_someday_maybe_items_by_user(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch someday/maybe items: {e}"))?;

    Ok(items)
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteSomedayMaybeError {
    #[error("Someday/maybe item not found")]
    NotFound,
    #[error("Not authorized to delete this someday/maybe item")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "delete_someday_maybe_item",
    skip(pool),
    fields(item_id = %item_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn delete_someday_maybe_item(
    pool: &PgPool,
    item_id: &SomedayMaybeId,
    user_id: &UserId,
) -> Result<(), DeleteSomedayMaybeError> {
    let item = someday_maybe_repository::find_someday_maybe_item_by_id(pool, item_id)
        .await
        .map_err(|e| DeleteSomedayMaybeError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(DeleteSomedayMaybeError::NotFound)?;

    if item.user_id() != user_id {
        return Err(DeleteSomedayMaybeError::Unauthorized);
    }

    someday_maybe_repository::delete_someday_maybe_item(pool, item_id)
        .await
        .map_err(|e| {
            DeleteSomedayMaybeError::Unexpected(anyhow::anyhow!(
                "Failed to delete someday/maybe item: {e}"
            ))
        })?;

    tracing::info!("Someday/maybe item deleted");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateSomedayMaybeError {
    #[error("Someday/maybe item not found")]
    NotFound,
    #[error("Not authorized to edit this someday/maybe item")]
    Unauthorized,
    #[error("Invalid title: {0}")]
    InvalidTitle(TodoTitleError),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

#[tracing::instrument(
    name = "update_someday_maybe_title",
    skip(pool, new_title),
    fields(item_id = %item_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn update_someday_maybe_title(
    pool: &PgPool,
    item_id: &SomedayMaybeId,
    user_id: &UserId,
    new_title: String,
) -> Result<(), UpdateSomedayMaybeError> {
    let item = someday_maybe_repository::find_someday_maybe_item_by_id(pool, item_id)
        .await
        .map_err(|e| UpdateSomedayMaybeError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(UpdateSomedayMaybeError::NotFound)?;

    if item.user_id() != user_id {
        return Err(UpdateSomedayMaybeError::Unauthorized);
    }

    let title = TodoTitle::parse(new_title).map_err(UpdateSomedayMaybeError::InvalidTitle)?;

    someday_maybe_repository::update_someday_maybe_title(pool, item_id, &title)
        .await
        .map_err(|e| {
            UpdateSomedayMaybeError::Unexpected(anyhow::anyhow!(
                "Failed to update someday/maybe item: {e}"
            ))
        })?;

    tracing::info!("Someday/maybe item updated");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ActivateSomedayMaybeError {
    #[error("Someday/maybe item not found")]
    NotFound,
    #[error("Not authorized to activate this someday/maybe item")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "activate_someday_maybe_item",
    skip(pool),
    fields(item_id = %item_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn activate_someday_maybe_item(
    pool: &PgPool,
    item_id: &SomedayMaybeId,
    user_id: &UserId,
) -> Result<InboxItem, ActivateSomedayMaybeError> {
    let mut tx = pool.begin().await.map_err(|e| {
        ActivateSomedayMaybeError::Unexpected(anyhow::anyhow!("Failed to begin transaction: {e}"))
    })?;

    let item = someday_maybe_repository::find_someday_maybe_item_by_id(&mut *tx, item_id)
        .await
        .map_err(|e| ActivateSomedayMaybeError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(ActivateSomedayMaybeError::NotFound)?;

    if item.user_id() != user_id {
        return Err(ActivateSomedayMaybeError::Unauthorized);
    }

    let inbox_item = InboxItem::new(user_id.clone(), item.title().clone());

    inbox_repository::insert_inbox_item(&mut *tx, &inbox_item)
        .await
        .map_err(|e| {
            ActivateSomedayMaybeError::Unexpected(anyhow::anyhow!(
                "Failed to insert inbox item: {e}"
            ))
        })?;

    someday_maybe_repository::delete_someday_maybe_item(&mut *tx, item_id)
        .await
        .map_err(|e| {
            ActivateSomedayMaybeError::Unexpected(anyhow::anyhow!(
                "Failed to delete someday/maybe item: {e}"
            ))
        })?;

    tx.commit().await.map_err(|e| {
        ActivateSomedayMaybeError::Unexpected(anyhow::anyhow!("Failed to commit transaction: {e}"))
    })?;

    tracing::info!("Someday/maybe item activated (moved to inbox)");
    Ok(inbox_item)
}
