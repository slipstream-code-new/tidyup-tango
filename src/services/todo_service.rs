use sqlx::PgPool;

use crate::domain::{TodoItem, TodoTitle, TodoTitleError, UserId};
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
