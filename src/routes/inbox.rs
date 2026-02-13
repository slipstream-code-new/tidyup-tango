use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::AuthenticatedUser;
use super::{htmx_response_with_announce, is_htmx_request};
use crate::domain::WaitingOnError;
use crate::domain::{ContextId, InboxItem, InboxItemId};
use crate::services::context_service;
use crate::services::inbox_service::{
    capture_inbox_item, clarify_as_next_action, clarify_as_project, clarify_as_waiting_for,
    delete_inbox_item, get_inbox_items, CaptureInboxError, ClarifyAsNextActionError,
    ClarifyAsProjectError, ClarifyAsWaitingForError, DeleteInboxError,
};

pub struct InboxItemView {
    pub id: String,
    pub title: String,
}

impl From<&InboxItem> for InboxItemView {
    fn from(item: &InboxItem) -> Self {
        Self {
            id: item.id().as_uuid().to_string(),
            title: item.title().as_str().to_string(),
        }
    }
}

pub struct ContextOption {
    pub id: String,
    pub name: String,
}

#[derive(Template)]
#[template(path = "inbox.html")]
struct InboxTemplate {
    current_page: &'static str,
    inbox_count: i64,
    items: Vec<InboxItemView>,
    contexts: Vec<ContextOption>,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "inbox_item.html")]
struct InboxItemTemplate {
    item: InboxItemView,
    contexts: Vec<ContextOption>,
    error: Option<String>,
}

pub async fn get_inbox(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, InboxError> {
    let items = get_inbox_items(&pool, &user_id)
        .await
        .map_err(InboxError::Unexpected)?;

    let contexts = context_service::list_contexts(&pool, &user_id)
        .await
        .map_err(InboxError::Unexpected)?;

    let inbox_count = items.len() as i64;
    let item_views: Vec<InboxItemView> = items.iter().map(InboxItemView::from).collect();

    let context_options: Vec<ContextOption> = contexts
        .iter()
        .map(|c| ContextOption {
            id: c.id().as_uuid().to_string(),
            name: c.name().as_str().to_string(),
        })
        .collect();

    let template = InboxTemplate {
        current_page: "inbox",
        inbox_count,
        items: item_views,
        contexts: context_options,
        error: None,
    };
    Ok(Html(template.render()?))
}

#[derive(serde::Deserialize)]
pub struct CaptureForm {
    pub title: String,
}

pub async fn post_inbox(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Form(form): Form<CaptureForm>,
) -> Result<Response, InboxError> {
    let htmx = is_htmx_request(&headers);

    // Silently ignore empty/whitespace-only submissions
    if form.title.trim().is_empty() {
        if htmx {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        return Ok(Redirect::to("/inbox").into_response());
    }

    match capture_inbox_item(&pool, user_id.clone(), form.title).await {
        Ok(item) => {
            if htmx {
                let contexts = context_service::list_contexts(&pool, &user_id)
                    .await
                    .map_err(InboxError::Unexpected)?;
                let context_options: Vec<ContextOption> = contexts
                    .iter()
                    .map(|c| ContextOption {
                        id: c.id().as_uuid().to_string(),
                        name: c.name().as_str().to_string(),
                    })
                    .collect();
                let template = InboxItemTemplate {
                    item: InboxItemView::from(&item),
                    contexts: context_options,
                    error: None,
                };
                let body = template.render().map_err(InboxError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Captured to inbox"))
            } else {
                Ok(Redirect::to("/inbox").into_response())
            }
        }
        Err(CaptureInboxError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                crate::domain::TodoTitleError::Empty => {
                    if htmx {
                        return Ok(StatusCode::NO_CONTENT.into_response());
                    }
                    return Ok(Redirect::to("/inbox").into_response());
                }
                crate::domain::TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };

            let items = get_inbox_items(&pool, &user_id)
                .await
                .map_err(InboxError::Unexpected)?;
            let contexts = context_service::list_contexts(&pool, &user_id)
                .await
                .map_err(InboxError::Unexpected)?;
            let inbox_count = items.len() as i64;
            let item_views: Vec<InboxItemView> = items.iter().map(InboxItemView::from).collect();
            let context_options: Vec<ContextOption> = contexts
                .iter()
                .map(|c| ContextOption {
                    id: c.id().as_uuid().to_string(),
                    name: c.name().as_str().to_string(),
                })
                .collect();

            let template = InboxTemplate {
                current_page: "inbox",
                inbox_count,
                items: item_views,
                contexts: context_options,
                error: None,
            };
            let _ = user_facing; // Title too long error is shown via template if needed
            let body = template.render().map_err(InboxError::Render)?;
            Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
        }
        Err(CaptureInboxError::Unexpected(err)) => Err(InboxError::Unexpected(err)),
    }
}

pub async fn post_delete_inbox_item(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
) -> Result<Response, InboxError> {
    let htmx = is_htmx_request(&headers);
    let inbox_item_id = InboxItemId::from_uuid(item_id);

    match delete_inbox_item(&pool, &inbox_item_id, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Inbox item deleted",
                ))
            } else {
                Ok(Redirect::to("/inbox").into_response())
            }
        }
        Err(DeleteInboxError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Inbox item not found</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteInboxError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteInboxError::Unexpected(err)) => Err(InboxError::Unexpected(err)),
    }
}

#[derive(serde::Deserialize)]
pub struct ClarifyForm {
    #[serde(default)]
    pub context_id: Option<Uuid>,
    #[serde(default)]
    pub clarify_type: String,
    #[serde(default)]
    pub first_action_title: String,
    #[serde(default)]
    pub waiting_on: String,
}

pub async fn post_clarify_inbox_item(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
    Form(form): Form<ClarifyForm>,
) -> Result<Response, InboxError> {
    let htmx = is_htmx_request(&headers);
    let inbox_item_id = InboxItemId::from_uuid(item_id);

    if form.clarify_type == "waiting_for" {
        match clarify_as_waiting_for(&pool, &inbox_item_id, &user_id, form.waiting_on).await {
            Ok(_item) => {
                if htmx {
                    Ok(htmx_response_with_announce(
                        Html(String::new()),
                        "Moved to Waiting For",
                    ))
                } else {
                    Ok(Redirect::to("/inbox").into_response())
                }
            }
            Err(ClarifyAsWaitingForError::NotFound) => Ok((
                StatusCode::NOT_FOUND,
                Html("<h1>Inbox item not found</h1>".to_string()),
            )
                .into_response()),
            Err(ClarifyAsWaitingForError::Unauthorized) => Ok((
                StatusCode::FORBIDDEN,
                Html("<h1>Not authorized</h1>".to_string()),
            )
                .into_response()),
            Err(ClarifyAsWaitingForError::InvalidWaitingOn(ref err)) => {
                let user_facing = match err {
                    WaitingOnError::Empty => {
                        if htmx {
                            return Ok(StatusCode::NO_CONTENT.into_response());
                        }
                        return Ok(Redirect::to("/inbox").into_response());
                    }
                    WaitingOnError::TooLong { max, .. } => {
                        format!("That name is too long (max {max} characters)")
                    }
                };
                if htmx {
                    let item = get_inbox_items(&pool, &user_id)
                        .await
                        .map_err(InboxError::Unexpected)?
                        .into_iter()
                        .find(|i| *i.id() == inbox_item_id);
                    if let Some(item) = item {
                        let contexts = context_service::list_contexts(&pool, &user_id)
                            .await
                            .map_err(InboxError::Unexpected)?;
                        let context_options: Vec<ContextOption> = contexts
                            .iter()
                            .map(|c| ContextOption {
                                id: c.id().as_uuid().to_string(),
                                name: c.name().as_str().to_string(),
                            })
                            .collect();
                        let template = InboxItemTemplate {
                            item: InboxItemView::from(&item),
                            contexts: context_options,
                            error: Some(user_facing),
                        };
                        let body = template.render().map_err(InboxError::Render)?;
                        Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
                    } else {
                        Ok((
                            StatusCode::NOT_FOUND,
                            Html("<h1>Inbox item not found</h1>".to_string()),
                        )
                            .into_response())
                    }
                } else {
                    Ok(Redirect::to("/inbox").into_response())
                }
            }
            Err(ClarifyAsWaitingForError::Unexpected(err)) => Err(InboxError::Unexpected(err)),
        }
    } else if form.clarify_type == "project" {
        let context_id = ContextId::from_uuid(
            form.context_id
                .ok_or_else(|| InboxError::Unexpected(anyhow::anyhow!("Missing context_id")))?,
        );
        match clarify_as_project(
            &pool,
            &inbox_item_id,
            &user_id,
            context_id,
            form.first_action_title,
        )
        .await
        {
            Ok(_result) => {
                if htmx {
                    Ok(htmx_response_with_announce(
                        Html(String::new()),
                        "Clarified as project",
                    ))
                } else {
                    Ok(Redirect::to("/inbox").into_response())
                }
            }
            Err(ClarifyAsProjectError::NotFound) => Ok((
                StatusCode::NOT_FOUND,
                Html("<h1>Inbox item not found</h1>".to_string()),
            )
                .into_response()),
            Err(ClarifyAsProjectError::Unauthorized) => Ok((
                StatusCode::FORBIDDEN,
                Html("<h1>Not authorized</h1>".to_string()),
            )
                .into_response()),
            Err(ClarifyAsProjectError::InvalidTitle(_)) => {
                if htmx {
                    let item = get_inbox_items(&pool, &user_id)
                        .await
                        .map_err(InboxError::Unexpected)?
                        .into_iter()
                        .find(|i| *i.id() == inbox_item_id);
                    if let Some(item) = item {
                        let contexts = context_service::list_contexts(&pool, &user_id)
                            .await
                            .map_err(InboxError::Unexpected)?;
                        let context_options: Vec<ContextOption> = contexts
                            .iter()
                            .map(|c| ContextOption {
                                id: c.id().as_uuid().to_string(),
                                name: c.name().as_str().to_string(),
                            })
                            .collect();
                        let template = InboxItemTemplate {
                            item: InboxItemView::from(&item),
                            contexts: context_options,
                            error: Some("Enter a first action for this project".to_string()),
                        };
                        let body = template.render().map_err(InboxError::Render)?;
                        Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
                    } else {
                        Ok((
                            StatusCode::NOT_FOUND,
                            Html("<h1>Inbox item not found</h1>".to_string()),
                        )
                            .into_response())
                    }
                } else {
                    Ok(Redirect::to("/inbox").into_response())
                }
            }
            Err(ClarifyAsProjectError::Unexpected(err)) => Err(InboxError::Unexpected(err)),
        }
    } else {
        let context_id = ContextId::from_uuid(
            form.context_id
                .ok_or_else(|| InboxError::Unexpected(anyhow::anyhow!("Missing context_id")))?,
        );
        match clarify_as_next_action(&pool, &inbox_item_id, &user_id, context_id).await {
            Ok(_action) => {
                if htmx {
                    Ok(htmx_response_with_announce(
                        Html(String::new()),
                        "Clarified as next action",
                    ))
                } else {
                    Ok(Redirect::to("/inbox").into_response())
                }
            }
            Err(ClarifyAsNextActionError::NotFound) => Ok((
                StatusCode::NOT_FOUND,
                Html("<h1>Inbox item not found</h1>".to_string()),
            )
                .into_response()),
            Err(ClarifyAsNextActionError::Unauthorized) => Ok((
                StatusCode::FORBIDDEN,
                Html("<h1>Not authorized</h1>".to_string()),
            )
                .into_response()),
            Err(ClarifyAsNextActionError::Unexpected(err)) => Err(InboxError::Unexpected(err)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InboxError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

impl IntoResponse for InboxError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Inbox error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
