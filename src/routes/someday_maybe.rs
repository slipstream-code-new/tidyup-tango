use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::AuthenticatedUser;
use super::{htmx_response_with_announce, is_htmx_request};
use crate::domain::{SomedayMaybeId, TodoTitleError};
use crate::services::inbox_service;
use crate::services::someday_maybe_service::{
    self, ActivateSomedayMaybeError, AddSomedayMaybeError, DeleteSomedayMaybeError,
    UpdateSomedayMaybeError,
};

pub struct SomedayMaybeView {
    pub id: String,
    pub title: String,
    pub date_added: String,
    pub date_added_iso: String,
}

#[derive(Template)]
#[template(path = "someday_maybe.html")]
struct SomedayMaybeTemplate {
    current_page: &'static str,
    inbox_count: i64,
    items: Vec<SomedayMaybeView>,
    error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "someday_maybe_item.html")]
struct SomedayMaybeItemTemplate {
    item: SomedayMaybeView,
}

#[derive(Template)]
#[template(path = "someday_maybe_edit.html")]
struct SomedayMaybeEditTemplate {
    item: SomedayMaybeView,
}

fn build_view(item: &crate::domain::SomedayMaybeItem) -> SomedayMaybeView {
    SomedayMaybeView {
        id: item.id().as_uuid().to_string(),
        title: item.title().as_str().to_string(),
        date_added: item.created_at().format("%b %d, %Y").to_string(),
        date_added_iso: item.created_at().to_rfc3339(),
    }
}

pub async fn get_someday_maybe(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Response, SomedayMaybeError> {
    let inbox_count = inbox_service::get_inbox_count(&pool, &user_id)
        .await
        .map_err(SomedayMaybeError::Unexpected)?;

    let items = someday_maybe_service::list_someday_maybe_items(&pool, &user_id)
        .await
        .map_err(SomedayMaybeError::Unexpected)?;

    let item_views: Vec<SomedayMaybeView> = items.iter().map(build_view).collect();

    let template = SomedayMaybeTemplate {
        current_page: "someday_maybe",
        inbox_count,
        items: item_views,
        error_message: None,
    };
    Ok(Html(template.render()?).into_response())
}

#[derive(serde::Deserialize)]
pub struct AddSomedayMaybeForm {
    pub title: String,
}

pub async fn post_someday_maybe(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Form(form): Form<AddSomedayMaybeForm>,
) -> Result<Response, SomedayMaybeError> {
    let htmx = is_htmx_request(&headers);

    if form.title.trim().is_empty() {
        if htmx {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        return Ok(Redirect::to("/someday-maybe").into_response());
    }

    match someday_maybe_service::add_someday_maybe_item(&pool, user_id.clone(), form.title).await {
        Ok(item) => {
            if htmx {
                let template = SomedayMaybeItemTemplate {
                    item: build_view(&item),
                };
                let body = template.render().map_err(SomedayMaybeError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Idea parked"))
            } else {
                Ok(Redirect::to("/someday-maybe").into_response())
            }
        }
        Err(AddSomedayMaybeError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                TodoTitleError::Empty => {
                    if htmx {
                        return Ok(StatusCode::NO_CONTENT.into_response());
                    }
                    return Ok(Redirect::to("/someday-maybe").into_response());
                }
                TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };
            render_someday_maybe_with_error(&pool, &user_id, &user_facing).await
        }
        Err(AddSomedayMaybeError::Unexpected(err)) => Err(SomedayMaybeError::Unexpected(err)),
    }
}

async fn render_someday_maybe_with_error(
    pool: &PgPool,
    user_id: &crate::domain::UserId,
    error_message: &str,
) -> Result<Response, SomedayMaybeError> {
    let inbox_count = inbox_service::get_inbox_count(pool, user_id)
        .await
        .map_err(SomedayMaybeError::Unexpected)?;

    let items = someday_maybe_service::list_someday_maybe_items(pool, user_id)
        .await
        .map_err(SomedayMaybeError::Unexpected)?;

    let item_views: Vec<SomedayMaybeView> = items.iter().map(build_view).collect();

    let template = SomedayMaybeTemplate {
        current_page: "someday_maybe",
        inbox_count,
        items: item_views,
        error_message: Some(error_message.to_string()),
    };
    let body = template.render().map_err(SomedayMaybeError::Render)?;
    Ok((StatusCode::UNPROCESSABLE_ENTITY, Html(body)).into_response())
}

pub async fn post_activate_someday_maybe(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
) -> Result<Response, SomedayMaybeError> {
    let htmx = is_htmx_request(&headers);
    let sm_id = SomedayMaybeId::from_uuid(item_id);

    match someday_maybe_service::activate_someday_maybe_item(&pool, &sm_id, &user_id).await {
        Ok(_inbox_item) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Moved to Inbox",
                ))
            } else {
                Ok(Redirect::to("/someday-maybe").into_response())
            }
        }
        Err(ActivateSomedayMaybeError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Someday/maybe item not found</h1>".to_string()),
        )
            .into_response()),
        Err(ActivateSomedayMaybeError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(ActivateSomedayMaybeError::Unexpected(err)) => Err(SomedayMaybeError::Unexpected(err)),
    }
}

pub async fn post_delete_someday_maybe(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
) -> Result<Response, SomedayMaybeError> {
    let htmx = is_htmx_request(&headers);
    let sm_id = SomedayMaybeId::from_uuid(item_id);

    match someday_maybe_service::delete_someday_maybe_item(&pool, &sm_id, &user_id).await {
        Ok(()) => {
            if htmx {
                Ok(htmx_response_with_announce(
                    Html(String::new()),
                    "Item deleted",
                ))
            } else {
                Ok(Redirect::to("/someday-maybe").into_response())
            }
        }
        Err(DeleteSomedayMaybeError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Someday/maybe item not found</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteSomedayMaybeError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(DeleteSomedayMaybeError::Unexpected(err)) => Err(SomedayMaybeError::Unexpected(err)),
    }
}

pub async fn get_edit_someday_maybe(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
) -> Result<Response, SomedayMaybeError> {
    let sm_id = SomedayMaybeId::from_uuid(item_id);
    let item = crate::infrastructure::someday_maybe_repository::find_someday_maybe_item_by_id(
        &pool, &sm_id,
    )
    .await
    .map_err(|e| SomedayMaybeError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
    .ok_or_else(|| SomedayMaybeError::Unexpected(anyhow::anyhow!("Item not found")))?;

    if item.user_id() != &user_id {
        return Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response());
    }

    let htmx = is_htmx_request(&headers);

    if htmx {
        let template = SomedayMaybeEditTemplate {
            item: build_view(&item),
        };
        let body = template.render().map_err(SomedayMaybeError::Render)?;
        Ok(Html(body).into_response())
    } else {
        Ok(Redirect::to("/someday-maybe").into_response())
    }
}

#[derive(serde::Deserialize)]
pub struct EditSomedayMaybeForm {
    pub title: String,
}

pub async fn post_edit_someday_maybe(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
    Form(form): Form<EditSomedayMaybeForm>,
) -> Result<Response, SomedayMaybeError> {
    let htmx = is_htmx_request(&headers);
    let sm_id = SomedayMaybeId::from_uuid(item_id);

    match someday_maybe_service::update_someday_maybe_title(&pool, &sm_id, &user_id, form.title)
        .await
    {
        Ok(()) => {
            if htmx {
                let item =
                    crate::infrastructure::someday_maybe_repository::find_someday_maybe_item_by_id(
                        &pool, &sm_id,
                    )
                    .await
                    .map_err(|e| {
                        SomedayMaybeError::Unexpected(anyhow::anyhow!("Database error: {e}"))
                    })?
                    .ok_or_else(|| {
                        SomedayMaybeError::Unexpected(anyhow::anyhow!("Item not found"))
                    })?;

                let template = SomedayMaybeItemTemplate {
                    item: build_view(&item),
                };
                let body = template.render().map_err(SomedayMaybeError::Render)?;
                Ok(htmx_response_with_announce(Html(body), "Item updated"))
            } else {
                Ok(Redirect::to("/someday-maybe").into_response())
            }
        }
        Err(UpdateSomedayMaybeError::NotFound) => Ok((
            StatusCode::NOT_FOUND,
            Html("<h1>Someday/maybe item not found</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateSomedayMaybeError::Unauthorized) => Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response()),
        Err(UpdateSomedayMaybeError::InvalidTitle(title_err)) => {
            let user_facing = match title_err {
                TodoTitleError::Empty => "Title cannot be empty".to_string(),
                TodoTitleError::TooLong { max, .. } => {
                    format!("That title is too long (max {max} characters)")
                }
            };
            render_someday_maybe_with_error(&pool, &user_id, &user_facing).await
        }
        Err(UpdateSomedayMaybeError::Unexpected(err)) => Err(SomedayMaybeError::Unexpected(err)),
    }
}

pub async fn get_someday_maybe_item(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(item_id): Path<Uuid>,
) -> Result<Response, SomedayMaybeError> {
    let sm_id = SomedayMaybeId::from_uuid(item_id);
    let item = crate::infrastructure::someday_maybe_repository::find_someday_maybe_item_by_id(
        &pool, &sm_id,
    )
    .await
    .map_err(|e| SomedayMaybeError::Unexpected(anyhow::anyhow!("Database error: {e}")))?
    .ok_or_else(|| SomedayMaybeError::Unexpected(anyhow::anyhow!("Item not found")))?;

    if item.user_id() != &user_id {
        return Ok((
            StatusCode::FORBIDDEN,
            Html("<h1>Not authorized</h1>".to_string()),
        )
            .into_response());
    }

    let template = SomedayMaybeItemTemplate {
        item: build_view(&item),
    };
    let body = template.render().map_err(SomedayMaybeError::Render)?;
    Ok(Html(body).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum SomedayMaybeError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

impl IntoResponse for SomedayMaybeError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "Someday/maybe error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}
