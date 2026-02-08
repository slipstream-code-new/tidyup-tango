use sqlx::PgPool;

use crate::domain::{TodoItem, TodoItemId, TodoTitle, TodoTitleError, UserId};
use crate::infrastructure::todo_repository;

#[derive(Debug, thiserror::Error)]
pub enum AddTodoError {
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "add_todo",
    skip(pool, title),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn add_todo(
    pool: &PgPool,
    user_id: UserId,
    title: String,
) -> Result<TodoItem, AddTodoError> {
    let title = TodoTitle::parse(title)?;
    let item = TodoItem::new_pending(user_id, title);

    todo_repository::insert_todo(pool, &item)
        .await
        .map_err(|e| AddTodoError::Unexpected(anyhow::anyhow!("Failed to insert todo: {e}")))?;

    tracing::info!("Todo added");
    Ok(item)
}

#[tracing::instrument(
    name = "get_todos",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn get_todos(pool: &PgPool, user_id: &UserId) -> Result<Vec<TodoItem>, anyhow::Error> {
    let todos = todo_repository::find_todos_by_user(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch todos: {e}"))?;

    Ok(todos)
}

#[derive(Debug, thiserror::Error)]
pub enum ToggleTodoError {
    #[error("Todo not found")]
    NotFound,
    #[error("Not authorized to modify this todo")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "toggle_todo_completion",
    skip(pool),
    fields(todo_id = %todo_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn toggle_todo_completion(
    pool: &PgPool,
    todo_id: &TodoItemId,
    user_id: &UserId,
) -> Result<TodoItem, ToggleTodoError> {
    let item = todo_repository::find_todo_by_id(pool, todo_id)
        .await
        .map_err(|e| ToggleTodoError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(ToggleTodoError::NotFound)?;

    if item.user_id() != user_id {
        return Err(ToggleTodoError::Unauthorized);
    }

    let toggled = if item.is_completed() {
        tracing::info!("Uncompleting todo");
        item.uncomplete()
    } else {
        tracing::info!("Completing todo");
        item.complete()
    };

    let completed_at = match &toggled {
        TodoItem::Completed { completed_at, .. } => Some(*completed_at),
        TodoItem::Pending { .. } => None,
    };

    todo_repository::update_todo_completion(pool, todo_id, completed_at)
        .await
        .map_err(|e| {
            ToggleTodoError::Unexpected(anyhow::anyhow!("Failed to update completion: {e}"))
        })?;

    Ok(toggled)
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteTodoError {
    #[error("Todo not found")]
    NotFound,
    #[error("Not authorized to delete this todo")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "delete_todo",
    skip(pool),
    fields(todo_id = %todo_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn delete_todo(
    pool: &PgPool,
    todo_id: &TodoItemId,
    user_id: &UserId,
) -> Result<(), DeleteTodoError> {
    let item = todo_repository::find_todo_by_id(pool, todo_id)
        .await
        .map_err(|e| DeleteTodoError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(DeleteTodoError::NotFound)?;

    if item.user_id() != user_id {
        return Err(DeleteTodoError::Unauthorized);
    }

    todo_repository::delete_todo(pool, todo_id)
        .await
        .map_err(|e| DeleteTodoError::Unexpected(anyhow::anyhow!("Failed to delete todo: {e}")))?;

    tracing::info!("Todo deleted");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateTitleError {
    #[error("Todo not found")]
    NotFound,
    #[error("Not authorized to edit this todo")]
    Unauthorized,
    #[error("Invalid title: {0}")]
    InvalidTitle(#[from] TodoTitleError),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

#[tracing::instrument(
    name = "update_todo_title",
    skip(pool, new_title),
    fields(todo_id = %todo_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn update_todo_title(
    pool: &PgPool,
    todo_id: &TodoItemId,
    user_id: &UserId,
    new_title: String,
) -> Result<(), UpdateTitleError> {
    let item = todo_repository::find_todo_by_id(pool, todo_id)
        .await
        .map_err(|e| UpdateTitleError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(UpdateTitleError::NotFound)?;

    if item.user_id() != user_id {
        return Err(UpdateTitleError::Unauthorized);
    }

    let validated = TodoTitle::parse(new_title)?;

    todo_repository::update_todo_title(pool, todo_id, validated.as_str())
        .await
        .map_err(|e| {
            UpdateTitleError::Unexpected(anyhow::anyhow!("Failed to update title: {e}"))
        })?;

    tracing::info!("Todo title updated");
    Ok(())
}
