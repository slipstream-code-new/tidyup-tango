use sqlx::PgPool;

use crate::domain::{Context, ContextId, ContextName, ContextNameError, UserId};
use crate::infrastructure::context_repository;

#[derive(Debug, thiserror::Error)]
pub enum AddContextError {
    #[error("Invalid context name: {0}")]
    InvalidName(#[from] ContextNameError),
    #[error("A context with this name already exists")]
    DuplicateName,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "add_context",
    skip(pool, name),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn add_context(
    pool: &PgPool,
    user_id: UserId,
    name: String,
) -> Result<Context, AddContextError> {
    let name = ContextName::parse(name)?;

    let max_pos = context_repository::get_max_position(pool, &user_id)
        .await
        .map_err(|e| AddContextError::Unexpected(anyhow::anyhow!("Database error: {e}")))?;

    let context = Context::new(user_id, name, max_pos + 1);

    context_repository::insert_context(pool, &context)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                AddContextError::DuplicateName
            } else {
                AddContextError::Unexpected(anyhow::anyhow!("Failed to insert context: {e}"))
            }
        })?;

    tracing::info!("Context added");
    Ok(context)
}

#[tracing::instrument(
    name = "list_contexts",
    skip(pool),
    fields(user_id = %user_id.as_uuid())
)]
pub async fn list_contexts(pool: &PgPool, user_id: &UserId) -> Result<Vec<Context>, anyhow::Error> {
    let contexts = context_repository::find_contexts_by_user(pool, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch contexts: {e}"))?;

    Ok(contexts)
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateContextError {
    #[error("Invalid context name: {0}")]
    InvalidName(#[from] ContextNameError),
    #[error("Context not found")]
    NotFound,
    #[error("Not authorized to update this context")]
    Unauthorized,
    #[error("A context with this name already exists")]
    DuplicateName,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "update_context",
    skip(pool, new_name),
    fields(context_id = %context_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn update_context(
    pool: &PgPool,
    context_id: &ContextId,
    user_id: &UserId,
    new_name: String,
) -> Result<(), UpdateContextError> {
    let context = context_repository::find_context_by_id(pool, context_id)
        .await
        .map_err(|e| UpdateContextError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(UpdateContextError::NotFound)?;

    if context.user_id() != user_id {
        return Err(UpdateContextError::Unauthorized);
    }

    let name = ContextName::parse(new_name)?;

    context_repository::update_context_name(pool, context_id, &name)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                UpdateContextError::DuplicateName
            } else {
                UpdateContextError::Unexpected(anyhow::anyhow!("Failed to update context: {e}"))
            }
        })?;

    tracing::info!("Context updated");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteContextError {
    #[error("Context not found")]
    NotFound,
    #[error("Not authorized to delete this context")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "delete_context",
    skip(pool),
    fields(context_id = %context_id.as_uuid(), user_id = %user_id.as_uuid())
)]
pub async fn delete_context(
    pool: &PgPool,
    context_id: &ContextId,
    user_id: &UserId,
) -> Result<(), DeleteContextError> {
    let context = context_repository::find_context_by_id(pool, context_id)
        .await
        .map_err(|e| DeleteContextError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or(DeleteContextError::NotFound)?;

    if context.user_id() != user_id {
        return Err(DeleteContextError::Unauthorized);
    }

    context_repository::delete_context(pool, context_id)
        .await
        .map_err(|e| {
            DeleteContextError::Unexpected(anyhow::anyhow!("Failed to delete context: {e}"))
        })?;

    tracing::info!("Context deleted");
    Ok(())
}

/// Seeds the 5 default GTD contexts for a new user.
pub async fn seed_default_contexts(pool: &PgPool, user_id: &UserId) -> Result<(), anyhow::Error> {
    let defaults = ["@computer", "@home", "@errands", "@phone", "@anywhere"];

    for (position, name) in defaults.iter().enumerate() {
        let context_name =
            ContextName::parse(name.to_string()).expect("default context names are valid");
        let context = Context::new(user_id.clone(), context_name, position as i32);
        context_repository::insert_context(pool, &context)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to seed context '{}': {}", name, e))?;
    }

    tracing::info!(user_id = %user_id.as_uuid(), "Default contexts seeded");
    Ok(())
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        db_err.code().as_deref() == Some("23505")
    } else {
        false
    }
}
