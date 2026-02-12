use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::AuthenticatedUser;
use super::{htmx_response_with_announce, is_htmx_request};
use crate::domain::{Context, ContextId, ContextNameError};
use crate::services::context_service::{
    add_context, delete_context, list_contexts, update_context, AddContextError,
    DeleteContextError, UpdateContextError,
};
use crate::services::inbox_service;

pub struct ContextView {
    pub id: String,
    pub name: String,
}

impl From<&Context> for ContextView {
    fn from(ctx: &Context) -> Self {
        Self {
            id: ctx.id().as_uuid().to_string(),
            name: ctx.name().as_str().to_string(),
        }
    }
}

#[derive(Template)]
#[template(path = "contexts.html")]
struct ContextsTemplate {
    current_page: &'static str,
    inbox_count: i64,
    contexts: Vec<ContextView>,
    error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "context_item.html")]
struct ContextItemTemplate {
    context: ContextView,
}

#[derive(Template)]
#[template(path = "context_edit.html")]
struct ContextEditTemplate {
    context: ContextView,
}

pub async fn get_contexts(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ContextError> {
    let contexts = list_contexts(&pool, &user_id)
        .await
        .map_err(ContextError::Unexpected)?;

    let inbox_count = inbox_service::get_inbox_count(&pool, &user_id)
        .await
        .map_err(ContextError::Unexpected)?;

    let context_views: Vec<ContextView> = contexts.iter().map(ContextView::from).collect();

    let template = ContextsTemplate {
        current_page: "contexts",
        inbox_count,
        contexts: context_views,
        error_message: None,
    };
    Ok(Html(template.render()?))
}

#[derive(serde::Deserialize)]
pub struct AddContextForm {
    pub name: String,
}

pub async fn post_context(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Form(form): Form<AddContextForm>,
) -> Result<Response, ContextError> {
    let htmx = is_htmx_request(&headers);

    if form.name.trim().is_empty() {
        if htmx {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        return Ok(Redirect::to("/contexts").into_response());
    }

    match add_context(&pool, user_id.clone(), form.name).await {
        Ok(context) => {
            if htmx {
                let template = ContextItemTemplate {
                    context: ContextView::from(&context),
                };
                let body = template.render().map_err(ContextError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Context added"))
            } else {
                Ok(Redirect::to("/contexts").into_response())
            }
        }
        Err(AddContextError::InvalidName(name_err)) => {
            let user_facing = match name_err {
                ContextNameError::Empty => {
                    if htmx {
                        return Ok(StatusCode::NO_CONTENT.into_response());
                    }
                    return Ok(Redirect::to("/contexts").into_response());
                }
                ContextNameError::TooLong { max, .. } => {
                    format!("That name is too long (max {max} characters)")
                }
            };

            render_contexts_with_error(&pool, &user_id, &user_facing).await
        }
        Err(AddContextError::DuplicateName) => {
            render_contexts_with_error(&pool, &user_id, "A context with that name already exists")
                .await
        }
        Err(AddContextError::Unexpected(err)) => Err(ContextError::Unexpected(err)),
    }
}

async fn render_contexts_with_error(
    pool: &PgPool,
    user_id: &crate::domain::UserId,
    error_message: &str,
) -> Result<Response, ContextError> {
    let contexts = list_contexts(pool, user_id)
        .await
        .map_err(ContextError::Unexpected)?;

    let inbox_count = inbox_service::get_inbox_count(pool, user_id)
        .await
        .map_err(ContextError::Unexpected)?;

    let context_views: Vec<ContextView> = contexts.iter().map(ContextView::from).collect();

    let template = ContextsTemplate {
        current_page: "contexts",
        inbox_count,
        contexts: context_views,
        error_message: Some(error_message.to_string()),
    };
    let body = template.render().map_err(ContextError::Render)?;
    Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
}

pub async fn get_edit_context(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(context_id): Path<Uuid>,
) -> Result<Response, ContextError> {
    let ctx_id = ContextId::from_uuid(context_id);
    let context = crate::infrastructure::context_repository::find_context_by_id(&pool, &ctx_id)
        .await
        .map_err(|e| ContextError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
        .ok_or_else(|| ContextError::Unexpected(anyhow::anyhow!("Context not found")))?;

    if context.user_id() != &user_id {
        return Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response());
    }

    let htmx = is_htmx_request(&headers);

    if htmx {
        let template = ContextEditTemplate {
            context: ContextView::from(&context),
        };
        let body = template.render().map_err(ContextError::Render)?;
        Ok(Html(body).into_response())
    } else {
        // Full page -- redirect to contexts page for non-HTMX
        Ok(Redirect::to("/contexts").into_response())
    }
}

#[derive(serde::Deserialize)]
pub struct EditContextForm {
    pub name: String,
}

pub async fn post_edit_context(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(context_id): Path<Uuid>,
    Form(form): Form<EditContextForm>,
) -> Result<Response, ContextError> {
    let htmx = is_htmx_request(&headers);
    let ctx_id = ContextId::from_uuid(context_id);

    match update_context(&pool, &ctx_id, &user_id, form.name).await {
        Ok(()) => {
            if htmx {
                let context =
                    crate::infrastructure::context_repository::find_context_by_id(&pool, &ctx_id)
                        .await
                        .map_err(|e| {
                            ContextError::Unexpected(anyhow::anyhow!("Database error: {e}"))
                        })?
                        .ok_or_else(|| {
                            ContextError::Unexpected(anyhow::anyhow!("Context not found"))
                        })?;

                let template = ContextItemTemplate {
                    context: ContextView::from(&context),
                };
                let body = template.render().map_err(ContextError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Context updated"))
            } else {
                Ok(Redirect::to("/contexts").into_response())
            }
        }
        Err(UpdateContextError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Context not found</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateContextError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateContextError::InvalidName(name_err)) => {
            let user_facing = match name_err {
                ContextNameError::Empty => "Context name cannot be empty".to_string(),
                ContextNameError::TooLong { max, .. } => {
                    format!("That name is too long (max {max} characters)")
                }
            };
            render_contexts_with_error(&pool, &user_id, &user_facing).await
        }
        Err(UpdateContextError::DuplicateName) => {
            render_contexts_with_error(&pool, &user_id, "A context with that name already exists")
                .await
        }
        Err(UpdateContextError::Unexpected(err)) => Err(ContextError::Unexpected(err)),
    }
}

pub async fn post_delete_context(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(context_id): Path<Uuid>,
) -> Result<Response, ContextError> {
    let htmx = is_htmx_request(&headers);
    let ctx_id = ContextId::from_uuid(context_id);

    match delete_context(&pool, &ctx_id, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Context deleted",
                ))
            } else {
                Ok(Redirect::to("/contexts").into_response())
            }
        }
        Err(DeleteContextError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Context not found</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteContextError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteContextError::Unexpected(err)) => Err(ContextError::Unexpected(err)),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

impl IntoResponse for ContextError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Context error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
