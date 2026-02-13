use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::AuthenticatedUser;
use super::{htmx_response_with_announce, is_htmx_request};
use crate::domain::{TodoTitleError, WaitingForId, WaitingOnError};
use crate::services::inbox_service;
use crate::services::waiting_for_service::{
    self, AddWaitingForError, DeleteWaitingForError, ResolveWaitingForError, UpdateWaitingForError,
};

pub struct WaitingForView {
    pub id: String,
    pub title: String,
    pub waiting_on: String,
    pub date_added: String,
    pub date_added_iso: String,
}

#[derive(Template)]
#[template(path = "waiting_for.html")]
struct WaitingForTemplate {
    current_page: &'static str,
    inbox_count: i64,
    items: Vec<WaitingForView>,
    error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "waiting_for_item.html")]
struct WaitingForItemTemplate {
    item: WaitingForView,
}

#[derive(Template)]
#[template(path = "waiting_for_edit.html")]
struct WaitingForEditTemplate {
    item: WaitingForView,
}

fn build_view(item: &crate::domain::WaitingForItem) -> WaitingForView {
    WaitingForView {
        id: item.id().as_uuid().to_string(),
        title: item.title().as_str().to_string(),
        waiting_on: item.waiting_on().as_str().to_string(),
        date_added: item.created_at().format("%b %d, %Y").to_string(),
        date_added_iso: item.created_at().to_rfc3339(),
    }
}

pub async fn get_waiting_for(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Response, WaitingForError> {
    let inbox_count = inbox_service::get_inbox_count(&pool, &user_id)
        .await
        .map_err(WaitingForError::Unexpected)?;

    let items = waiting_for_service::list_active_waiting_for_items(&pool, &user_id)
        .await
        .map_err(WaitingForError::Unexpected)?;

    let item_views: Vec<WaitingForView> = items.iter().map(build_view).collect();

    let template = WaitingForTemplate {
        current_page: "waiting_for",
        inbox_count,
        items: item_views,
        error_message: None,
    };
    Ok(Html(template.render()?).into_response())
}

#[derive(serde::Deserialize)]
pub struct AddWaitingForForm {
    pub title: String,
    pub waiting_on: String,
}

pub async fn post_waiting_for(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Form(form): Form<AddWaitingForForm>,
) -> Result<Response, WaitingForError> {
    let htmx = is_htmx_request(&headers);

    if form.title.trim().is_empty() {
        if htmx {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        return Ok(Redirect::to("/waiting-for").into_response());
    }

    if form.waiting_on.trim().is_empty() {
        if htmx {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        return Ok(Redirect::to("/waiting-for").into_response());
    }

    match waiting_for_service::add_waiting_for_item(
        &pool,
        user_id.clone(),
        form.title,
        form.waiting_on,
    )
    .await
    {
        Ok(item) => {
            if htmx {
                let template = WaitingForItemTemplate {
                    item: build_view(&item),
                };
                let body = template.render().map_err(WaitingForError::Render)?;
                Ok(htmx_response_with_announce(
                    Html(body),
                    "Waiting-for item added",
                ))
            } else {
                Ok(Redirect::to("/waiting-for").into_response())
            }
        }
        Err(AddWaitingForError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                TodoTitleError::Empty => {
                    if htmx {
                        return Ok(StatusCode::NO_CONTENT.into_response());
                    }
                    return Ok(Redirect::to("/waiting-for").into_response());
                }
                TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };
            render_waiting_for_with_error(&pool, &user_id, &user_facing).await
        }
        Err(AddWaitingForError::InvalidWaitingOn(person_err)) => {
            let user_facing = match person_err {
                WaitingOnError::Empty => {
                    if htmx {
                        return Ok(StatusCode::NO_CONTENT.into_response());
                    }
                    return Ok(Redirect::to("/waiting-for").into_response());
                }
                WaitingOnError::TooLong { max, .. } => {
                    format!("That name is too long (max {max} characters)")
                }
            };
            render_waiting_for_with_error(&pool, &user_id, &user_facing).await
        }
        Err(AddWaitingForError::Unexpected(err)) => Err(WaitingForError::Unexpected(err)),
    }
}

async fn render_waiting_for_with_error(
    pool: &PgPool,
    user_id: &crate::domain::UserId,
    error_message: &str,
) -> Result<Response, WaitingForError> {
    let inbox_count = inbox_service::get_inbox_count(pool, user_id)
        .await
        .map_err(WaitingForError::Unexpected)?;

    let items = waiting_for_service::list_active_waiting_for_items(pool, user_id)
        .await
        .map_err(WaitingForError::Unexpected)?;

    let item_views: Vec<WaitingForView> = items.iter().map(build_view).collect();

    let template = WaitingForTemplate {
        current_page: "waiting_for",
        inbox_count,
        items: item_views,
        error_message: Some(error_message.to_string()),
    };
    let body = template.render().map_err(WaitingForError::Render)?;
    Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
}

pub async fn post_complete_waiting_for(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
) -> Result<Response, WaitingForError> {
    let htmx = is_htmx_request(&headers);
    let wf_id = WaitingForId::from_uuid(item_id);

    match waiting_for_service::resolve_waiting_for_item(&pool, &wf_id, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Item received",
                ))
            } else {
                Ok(Redirect::to("/waiting-for").into_response())
            }
        }
        Err(ResolveWaitingForError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Waiting-for item not found</h1>".to_string()),
        )
            .into_response()),
        Err(ResolveWaitingForError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(ResolveWaitingForError::Unexpected(err)) => Err(WaitingForError::Unexpected(err)),
    }
}

pub async fn post_delete_waiting_for(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
) -> Result<Response, WaitingForError> {
    let htmx = is_htmx_request(&headers);
    let wf_id = WaitingForId::from_uuid(item_id);

    match waiting_for_service::delete_waiting_for_item(&pool, &wf_id, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Item deleted",
                ))
            } else {
                Ok(Redirect::to("/waiting-for").into_response())
            }
        }
        Err(DeleteWaitingForError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Waiting-for item not found</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteWaitingForError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteWaitingForError::Unexpected(err)) => Err(WaitingForError::Unexpected(err)),
    }
}

pub async fn get_edit_waiting_for(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
) -> Result<Response, WaitingForError> {
    let wf_id = WaitingForId::from_uuid(item_id);
    let item =
        crate::infrastructure::waiting_for_repository::find_waiting_for_item_by_id(&pool, &wf_id)
            .await
            .map_err(|e| WaitingForError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
            .ok_or_else(|| WaitingForError::Unexpected(anyhow::anyhow!("Item not found")))?;

    if item.user_id() != &user_id {
        return Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response());
    }

    let htmx = is_htmx_request(&headers);

    if htmx {
        let template = WaitingForEditTemplate {
            item: build_view(&item),
        };
        let body = template.render().map_err(WaitingForError::Render)?;
        Ok(Html(body).into_response())
    } else {
        Ok(Redirect::to("/waiting-for").into_response())
    }
}

#[derive(serde::Deserialize)]
pub struct EditWaitingForForm {
    pub title: String,
    pub waiting_on: String,
}

pub async fn post_edit_waiting_for(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
    Form(form): Form<EditWaitingForForm>,
) -> Result<Response, WaitingForError> {
    let htmx = is_htmx_request(&headers);
    let wf_id = WaitingForId::from_uuid(item_id);

    match waiting_for_service::update_waiting_for_item(
        &pool,
        &wf_id,
        &user_id,
        form.title,
        form.waiting_on,
    )
    .await
    {
        Ok(()) => {
            if htmx {
                let item =
                    crate::infrastructure::waiting_for_repository::find_waiting_for_item_by_id(
                        &pool, &wf_id,
                    )
                    .await
                    .map_err(|e| {
                        WaitingForError::Unexpected(anyhow::anyhow!("Database error: {e}"))
                    })?
                    .ok_or_else(|| {
                        WaitingForError::Unexpected(anyhow::anyhow!("Item not found"))
                    })?;

                let template = WaitingForItemTemplate {
                    item: build_view(&item),
                };
                let body = template.render().map_err(WaitingForError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Item updated"))
            } else {
                Ok(Redirect::to("/waiting-for").into_response())
            }
        }
        Err(UpdateWaitingForError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Waiting-for item not found</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateWaitingForError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateWaitingForError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                TodoTitleError::Empty => "Title cannot be empty".to_string(),
                TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };
            render_waiting_for_with_error(&pool, &user_id, &user_facing).await
        }
        Err(UpdateWaitingForError::InvalidWaitingOn(person_err)) => {
            let user_facing = match person_err {
                WaitingOnError::Empty => "Who or what cannot be empty".to_string(),
                WaitingOnError::TooLong { max, .. } => {
                    format!("That name is too long (max {max} characters)")
                }
            };
            render_waiting_for_with_error(&pool, &user_id, &user_facing).await
        }
        Err(UpdateWaitingForError::Unexpected(err)) => Err(WaitingForError::Unexpected(err)),
    }
}

pub async fn get_waiting_for_item(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(item_id): Path<Uuid>,
) -> Result<Response, WaitingForError> {
    let wf_id = WaitingForId::from_uuid(item_id);
    let item =
        crate::infrastructure::waiting_for_repository::find_waiting_for_item_by_id(&pool, &wf_id)
            .await
            .map_err(|e| WaitingForError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
            .ok_or_else(|| WaitingForError::Unexpected(anyhow::anyhow!("Item not found")))?;

    if item.user_id() != &user_id {
        return Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response());
    }

    let template = WaitingForItemTemplate {
        item: build_view(&item),
    };
    let body = template.render().map_err(WaitingForError::Render)?;
    Ok(Html(body).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum WaitingForError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

impl IntoResponse for WaitingForError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Waiting-for error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
