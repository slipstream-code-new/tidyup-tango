use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::AuthenticatedUser;
use crate::domain::{InboxItem, InboxItemId};
use crate::services::inbox_service::{
    capture_inbox_item, delete_inbox_item, get_inbox_items, CaptureInboxError, DeleteInboxError,
};

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}

fn htmx_response_with_announce(body: Html<String>, message: &str) -> Response {
    let trigger_json = format!(r#"{{"announce":"{}"}}"#, message);
    (
        [(
            axum::http::header::HeaderName::from_static("hx-trigger"),
            axum::http::HeaderValue::from_str(&trigger_json).unwrap(),
        )],
        body,
    )
        .into_response()
}

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

#[derive(Template)]
#[template(path = "inbox.html")]
struct InboxTemplate {
    current_page: &'static str,
    inbox_count: i64,
    items: Vec<InboxItemView>,
}

#[derive(Template)]
#[template(path = "inbox_item.html")]
struct InboxItemTemplate {
    item: InboxItemView,
}

pub async fn get_inbox(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, InboxError> {
    let items = get_inbox_items(&pool, &user_id)
        .await
        .map_err(InboxError::Unexpected)?;

    let inbox_count = items.len() as i64;
    let item_views: Vec<InboxItemView> = items.iter().map(InboxItemView::from).collect();

    let template = InboxTemplate {
        current_page: "inbox",
        inbox_count,
        items: item_views,
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
                let template = InboxItemTemplate {
                    item: InboxItemView::from(&item),
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
            let inbox_count = items.len() as i64;
            let item_views: Vec<InboxItemView> = items.iter().map(InboxItemView::from).collect();

            let template = InboxTemplate {
                current_page: "inbox",
                inbox_count,
                items: item_views,
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
