use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::AuthenticatedUser;
use super::{htmx_response_with_announce, is_htmx_request};
use crate::domain::{Context, ContextId, NextAction, NextActionId, TodoTitleError};
use crate::services::context_service;
use crate::services::inbox_service;
use crate::services::next_action_service::{
    self, AddNextActionError, CompleteNextActionError, DeleteNextActionError,
    UpdateNextActionTitleError,
};

pub struct NextActionView {
    pub id: String,
    pub title: String,
    pub context_name: String,
}

pub struct ContextOption {
    pub id: String,
    pub name: String,
}

pub struct ContextGroup {
    pub context_name: String,
    pub context_id: String,
    pub actions: Vec<NextActionView>,
}

#[derive(Template)]
#[template(path = "next_actions.html")]
struct NextActionsTemplate {
    current_page: &'static str,
    inbox_count: i64,
    groups: Vec<ContextGroup>,
    contexts: Vec<ContextOption>,
    selected_context: Option<String>,
    error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "next_action_item.html")]
struct NextActionItemTemplate {
    action: NextActionView,
}

#[derive(Template)]
#[template(path = "next_action_list.html")]
struct NextActionListTemplate {
    groups: Vec<ContextGroup>,
    show_headings: bool,
}

#[derive(Template)]
#[template(path = "next_action_edit.html")]
struct NextActionEditTemplate {
    action: NextActionView,
}

fn build_action_view(action: &NextAction, contexts: &[Context]) -> NextActionView {
    let context_name = contexts
        .iter()
        .find(|c| c.id() == action.context_id())
        .map(|c| c.name().as_str().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    NextActionView {
        id: action.id().as_uuid().to_string(),
        title: action.title().as_str().to_string(),
        context_name,
    }
}

fn build_context_groups(
    actions: &[NextAction],
    contexts: &[Context],
    single_context: bool,
) -> Vec<ContextGroup> {
    if single_context {
        // When filtering by a single context, put all actions in one group
        // (the heading is redundant since the filter pill already shows it)
        let context_name = actions
            .first()
            .and_then(|a| {
                contexts
                    .iter()
                    .find(|c| c.id() == a.context_id())
                    .map(|c| c.name().as_str().to_string())
            })
            .unwrap_or_default();

        let context_id = actions
            .first()
            .map(|a| a.context_id().as_uuid().to_string())
            .unwrap_or_default();

        let action_views = actions
            .iter()
            .map(|a| build_action_view(a, contexts))
            .collect();

        return vec![ContextGroup {
            context_name,
            context_id,
            actions: action_views,
        }];
    }

    // "All" view: group actions by context, preserving context order
    let mut groups: Vec<ContextGroup> = Vec::new();

    for ctx in contexts {
        let ctx_actions: Vec<NextActionView> = actions
            .iter()
            .filter(|a| a.context_id() == ctx.id())
            .map(|a| build_action_view(a, contexts))
            .collect();

        if !ctx_actions.is_empty() {
            groups.push(ContextGroup {
                context_name: ctx.name().as_str().to_string(),
                context_id: ctx.id().as_uuid().to_string(),
                actions: ctx_actions,
            });
        }
    }

    groups
}

#[derive(serde::Deserialize)]
pub struct ContextFilter {
    pub context: Option<Uuid>,
}

pub async fn get_next_actions(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Query(filter): Query<ContextFilter>,
) -> Result<Response, NextActionError> {
    let htmx = is_htmx_request(&headers);

    let contexts = context_service::list_contexts(&pool, &user_id)
        .await
        .map_err(NextActionError::Unexpected)?;

    let actions = if let Some(ctx_uuid) = filter.context {
        let ctx_id = ContextId::from_uuid(ctx_uuid);
        next_action_service::list_active_next_actions_by_context(&pool, &user_id, &ctx_id)
            .await
            .map_err(NextActionError::Unexpected)?
    } else {
        next_action_service::list_active_next_actions(&pool, &user_id)
            .await
            .map_err(NextActionError::Unexpected)?
    };

    let show_headings = filter.context.is_none();
    let groups = build_context_groups(&actions, &contexts, filter.context.is_some());

    // HTMX filter requests: return just the action list fragment
    if htmx {
        let template = NextActionListTemplate {
            groups,
            show_headings,
        };
        let body = template.render().map_err(NextActionError::Render)?;
        return Ok(Html(body).into_response());
    }

    let inbox_count = inbox_service::get_inbox_count(&pool, &user_id)
        .await
        .map_err(NextActionError::Unexpected)?;

    let context_options: Vec<ContextOption> = contexts
        .iter()
        .map(|c| ContextOption {
            id: c.id().as_uuid().to_string(),
            name: c.name().as_str().to_string(),
        })
        .collect();

    let selected_context = filter.context.map(|c| c.to_string());

    let template = NextActionsTemplate {
        current_page: "next_actions",
        inbox_count,
        groups,
        contexts: context_options,
        selected_context,
        error_message: None,
    };
    Ok(Html(template.render()?).into_response())
}

#[derive(serde::Deserialize)]
pub struct AddNextActionForm {
    pub title: String,
    pub context_id: Uuid,
}

pub async fn post_next_action(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Form(form): Form<AddNextActionForm>,
) -> Result<Response, NextActionError> {
    let htmx = is_htmx_request(&headers);

    if form.title.trim().is_empty() {
        if htmx {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        return Ok(Redirect::to("/next-actions").into_response());
    }

    let context_id = ContextId::from_uuid(form.context_id);

    let contexts = context_service::list_contexts(&pool, &user_id)
        .await
        .map_err(NextActionError::Unexpected)?;

    match next_action_service::add_next_action(&pool, user_id.clone(), context_id, form.title).await
    {
        Ok(action) => {
            if htmx {
                let template = NextActionItemTemplate {
                    action: build_action_view(&action, &contexts),
                };
                let body = template.render().map_err(NextActionError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Next action added"))
            } else {
                Ok(Redirect::to("/next-actions").into_response())
            }
        }
        Err(AddNextActionError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                TodoTitleError::Empty => {
                    if htmx {
                        return Ok(StatusCode::NO_CONTENT.into_response());
                    }
                    return Ok(Redirect::to("/next-actions").into_response());
                }
                TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };

            render_next_actions_with_error(&pool, &user_id, &user_facing).await
        }
        Err(AddNextActionError::Unexpected(err)) => Err(NextActionError::Unexpected(err)),
    }
}

async fn render_next_actions_with_error(
    pool: &PgPool,
    user_id: &crate::domain::UserId,
    error_message: &str,
) -> Result<Response, NextActionError> {
    let inbox_count = inbox_service::get_inbox_count(pool, user_id)
        .await
        .map_err(NextActionError::Unexpected)?;

    let contexts = context_service::list_contexts(pool, user_id)
        .await
        .map_err(NextActionError::Unexpected)?;

    let actions = next_action_service::list_active_next_actions(pool, user_id)
        .await
        .map_err(NextActionError::Unexpected)?;

    let groups = build_context_groups(&actions, &contexts, false);

    let context_options: Vec<ContextOption> = contexts
        .iter()
        .map(|c| ContextOption {
            id: c.id().as_uuid().to_string(),
            name: c.name().as_str().to_string(),
        })
        .collect();

    let template = NextActionsTemplate {
        current_page: "next_actions",
        inbox_count,
        groups,
        contexts: context_options,
        selected_context: None,
        error_message: Some(error_message.to_string()),
    };
    let body = template.render().map_err(NextActionError::Render)?;
    Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
}

pub async fn post_complete_next_action(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(action_id): Path<Uuid>,
) -> Result<Response, NextActionError> {
    let htmx = is_htmx_request(&headers);
    let na_id = NextActionId::from_uuid(action_id);

    match next_action_service::complete_next_action(&pool, &na_id, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Action completed",
                ))
            } else {
                Ok(Redirect::to("/next-actions").into_response())
            }
        }
        Err(CompleteNextActionError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Next action not found</h1>".to_string()),
        )
            .into_response()),
        Err(CompleteNextActionError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(CompleteNextActionError::Unexpected(err)) => Err(NextActionError::Unexpected(err)),
    }
}

pub async fn post_delete_next_action(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(action_id): Path<Uuid>,
) -> Result<Response, NextActionError> {
    let htmx = is_htmx_request(&headers);
    let na_id = NextActionId::from_uuid(action_id);

    match next_action_service::delete_next_action(&pool, &na_id, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Next action deleted",
                ))
            } else {
                Ok(Redirect::to("/next-actions").into_response())
            }
        }
        Err(DeleteNextActionError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Next action not found</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteNextActionError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteNextActionError::Unexpected(err)) => Err(NextActionError::Unexpected(err)),
    }
}

pub async fn get_edit_next_action(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(action_id): Path<Uuid>,
) -> Result<Response, NextActionError> {
    let na_id = NextActionId::from_uuid(action_id);
    let action =
        crate::infrastructure::next_action_repository::find_next_action_by_id(&pool, &na_id)
            .await
            .map_err(|e| NextActionError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
            .ok_or_else(|| NextActionError::Unexpected(anyhow::anyhow!("Next action not found")))?;

    if action.user_id() != &user_id {
        return Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response());
    }

    let contexts = context_service::list_contexts(&pool, &user_id)
        .await
        .map_err(NextActionError::Unexpected)?;

    let htmx = is_htmx_request(&headers);

    if htmx {
        let template = NextActionEditTemplate {
            action: build_action_view(&action, &contexts),
        };
        let body = template.render().map_err(NextActionError::Render)?;
        Ok(Html(body).into_response())
    } else {
        Ok(Redirect::to("/next-actions").into_response())
    }
}

#[derive(serde::Deserialize)]
pub struct EditNextActionForm {
    pub title: String,
}

pub async fn post_edit_next_action(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(action_id): Path<Uuid>,
    Form(form): Form<EditNextActionForm>,
) -> Result<Response, NextActionError> {
    let htmx = is_htmx_request(&headers);
    let na_id = NextActionId::from_uuid(action_id);

    match next_action_service::update_next_action_title(&pool, &na_id, &user_id, form.title).await {
        Ok(()) => {
            if htmx {
                let action = crate::infrastructure::next_action_repository::find_next_action_by_id(
                    &pool, &na_id,
                )
                .await
                .map_err(|e| NextActionError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
                .ok_or_else(|| {
                    NextActionError::Unexpected(anyhow::anyhow!("Next action not found"))
                })?;

                let contexts = context_service::list_contexts(&pool, &user_id)
                    .await
                    .map_err(NextActionError::Unexpected)?;

                let template = NextActionItemTemplate {
                    action: build_action_view(&action, &contexts),
                };
                let body = template.render().map_err(NextActionError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Action updated"))
            } else {
                Ok(Redirect::to("/next-actions").into_response())
            }
        }
        Err(UpdateNextActionTitleError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Next action not found</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateNextActionTitleError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateNextActionTitleError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                TodoTitleError::Empty => "Title cannot be empty".to_string(),
                TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };
            render_next_actions_with_error(&pool, &user_id, &user_facing).await
        }
        Err(UpdateNextActionTitleError::Unexpected(err)) => Err(NextActionError::Unexpected(err)),
    }
}

pub async fn get_next_action_item(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(action_id): Path<Uuid>,
) -> Result<Response, NextActionError> {
    let na_id = NextActionId::from_uuid(action_id);
    let action =
        crate::infrastructure::next_action_repository::find_next_action_by_id(&pool, &na_id)
            .await
            .map_err(|e| NextActionError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
            .ok_or_else(|| NextActionError::Unexpected(anyhow::anyhow!("Next action not found")))?;

    if action.user_id() != &user_id {
        return Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response());
    }

    let contexts = context_service::list_contexts(&pool, &user_id)
        .await
        .map_err(NextActionError::Unexpected)?;

    let template = NextActionItemTemplate {
        action: build_action_view(&action, &contexts),
    };
    let body = template.render().map_err(NextActionError::Render)?;
    Ok(Html(body).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum NextActionError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

impl IntoResponse for NextActionError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Next action error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
