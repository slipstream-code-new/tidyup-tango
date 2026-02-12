use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;

use super::auth::AuthenticatedUser;
use crate::services::inbox_service;

#[derive(Template)]
#[template(path = "gtd_placeholder.html")]
struct GtdPlaceholderTemplate {
    current_page: &'static str,
    page_title: &'static str,
    heading: &'static str,
    description: &'static str,
    inbox_count: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum GtdPlaceholderError {
    #[error("Failed to render template")]
    Render(#[from] askama::Error),
    #[error(transparent)]
    Unexpected(anyhow::Error),
}

impl IntoResponse for GtdPlaceholderError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "GTD placeholder error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Something went wrong</h1>".to_string()),
        )
            .into_response()
    }
}

async fn render_placeholder(
    user_id: &crate::domain::UserId,
    pool: &PgPool,
    current_page: &'static str,
    page_title: &'static str,
    heading: &'static str,
    description: &'static str,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    let inbox_count = inbox_service::get_inbox_count(pool, user_id)
        .await
        .map_err(GtdPlaceholderError::Unexpected)?;

    let template = GtdPlaceholderTemplate {
        current_page,
        page_title,
        heading,
        description,
        inbox_count,
    };
    Ok(Html(template.render()?))
}

pub async fn get_projects(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    render_placeholder(
        &user_id,
        &pool,
        "projects",
        "Projects -- Todo List",
        "Projects",
        "Outcomes that need more than one action step.",
    )
    .await
}

pub async fn get_waiting_for(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    render_placeholder(
        &user_id,
        &pool,
        "waiting_for",
        "Waiting For -- Todo List",
        "Waiting For",
        "Items you're waiting on from others.",
    )
    .await
}

pub async fn get_someday_maybe(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    render_placeholder(
        &user_id,
        &pool,
        "someday_maybe",
        "Someday/Maybe -- Todo List",
        "Someday/Maybe",
        "Ideas and possibilities for the future.",
    )
    .await
}

pub async fn get_review(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, GtdPlaceholderError> {
    render_placeholder(
        &user_id,
        &pool,
        "review",
        "Weekly Review -- Todo List",
        "Weekly Review",
        "Keep your system current and trustworthy.",
    )
    .await
}
